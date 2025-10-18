<script lang="ts">
	interface FolderItem {
		id: string;
		path: string;
		name: string;
		handle?: FileSystemDirectoryHandle; // File System Access API용
	}

	let folders = $state<FolderItem[]>([]);
	let isDragging = $state(false);
	let message = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);

	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		isDragging = true;
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'copy';
		}
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		// 실제로 드롭 영역을 벗어났을 때만 isDragging을 false로 설정
		if (e.currentTarget === e.target) {
			isDragging = false;
		}
	}

	// 절대 경로 추출 시도 함수
	async function getAbsolutePath(entry: any, file: File | null): Promise<string> {
		// 방법 1: File 객체의 path 속성 (Electron 등에서 사용 가능)
		if (file && 'path' in file) {
			const filePath = (file as any).path;
			console.log('File.path 발견:', filePath);
			return filePath;
		}

		// 방법 2: webkitRelativePath 확인
		if (file && file.webkitRelativePath) {
			console.log('webkitRelativePath 발견:', file.webkitRelativePath);
			return file.webkitRelativePath;
		}

		// 방법 3: FileSystemEntry의 fullPath (가상 경로)
		if (entry && entry.fullPath) {
			console.log('entry.fullPath (가상 경로):', entry.fullPath);
			return entry.fullPath;
		}

		// 방법 4: 이름만 반환
		return entry ? entry.name : file?.name || 'Unknown';
	}

	async function handleDrop(e: DragEvent) {
		e.preventDefault();
		isDragging = false;

		if (!e.dataTransfer) return;

		console.log('=== Drop Event ===');
		console.log('DataTransfer:', e.dataTransfer);

		const items = e.dataTransfer.items;
		const files = e.dataTransfer.files;
		const validFolders: FolderItem[] = [];
		const rejectedFiles: string[] = [];

		console.log('Items:', items);
		console.log('Files:', files);

		// DataTransferItemList를 순회하며 폴더만 필터링
		for (let i = 0; i < items.length; i++) {
			const item = items[i];
			const file = files[i];

			console.log(`\n--- Item ${i} ---`);
			console.log('Item:', item);
			console.log('File:', file);
			console.log('File.path:', (file as any).path);

			if (item.kind === 'file') {
				const entry = item.webkitGetAsEntry();
				console.log('Entry:', entry);

				if (entry && entry.isDirectory) {
					// 중복 체크
					const absolutePath = await getAbsolutePath(entry, file);
					
					if (validFolders.findIndex(f => f.path === absolutePath) === -1 &&
							folders.findIndex(f => f.path === absolutePath) === -1) {
						// 폴더인 경우
						const folderItem: FolderItem = {
							id: `${Date.now()}-${i}`,
							path: absolutePath,
							name: entry.name
						};
						console.log('추가할 폴더:', folderItem);
						validFolders.push(folderItem);
					} else {
						console.log('중복된 폴더:', absolutePath);
					}
				} else if (entry && entry.isFile) {
					// 파일인 경우 (거부)
					rejectedFiles.push(entry.name);
				}
			}
		}

		// 결과 처리
		if (validFolders.length > 0) {
			folders = [...folders, ...validFolders];
			showMessage(
				`${validFolders.length}개의 폴더가 추가되었습니다.`,
				'success'
			);
		}

		if (rejectedFiles.length > 0) {
			showMessage(
				`${rejectedFiles.length}개의 파일은 추가되지 않았습니다. 폴더만 추가할 수 있습니다.`,
				'error'
			);
		}

		if (validFolders.length === 0 && rejectedFiles.length === 0) {
			showMessage('추가할 폴더가 없습니다.', 'info');
		}
	}

	// File System Access API를 사용한 디렉토리 선택
	async function selectDirectory() {
		try {
			// @ts-ignore - File System Access API
			if (!window.showDirectoryPicker) {
				showMessage(
					'이 브라우저는 디렉토리 선택 API를 지원하지 않습니다. Chrome 86+ 사용을 권장합니다.',
					'error'
				);
				return;
			}

			// @ts-ignore
			const dirHandle: FileSystemDirectoryHandle = await window.showDirectoryPicker({
				mode: 'read'
			});

			console.log('=== Directory Picker ===');
			console.log('Handle:', dirHandle);
			console.log('Name:', dirHandle.name);

			// 절대 경로를 얻을 수 있는지 확인
			let absolutePath = dirHandle.name;
			
			// 일부 환경에서는 경로 정보를 제공할 수 있음
			if ('getDirectoryHandle' in dirHandle) {
				try {
					// @ts-ignore
					const path = await dirHandle.resolve?.() || dirHandle.name;
					console.log('Resolved path:', path);
					absolutePath = Array.isArray(path) ? path.join('/') : path;
				} catch (e) {
					console.log('경로 resolve 실패:', e);
				}
			}

			// 중복 체크
			if (folders.findIndex(f => f.name === dirHandle.name && f.path === absolutePath) !== -1) {
				showMessage('이미 추가된 폴더입니다.', 'info');
				return;
			}

			const folderItem: FolderItem = {
				id: `${Date.now()}`,
				path: absolutePath,
				name: dirHandle.name,
				handle: dirHandle
			};

			folders = [...folders, folderItem];
			showMessage(
				`폴더 "${dirHandle.name}"가 추가되었습니다. (File System Access API)`,
				'success'
			);
		} catch (error: any) {
			if (error.name !== 'AbortError') {
				console.error('디렉토리 선택 오류:', error);
				showMessage('디렉토리 선택 중 오류가 발생했습니다.', 'error');
			}
		}
	}

	function removeFolder(id: string) {
		folders = folders.filter((f) => f.id !== id);
		showMessage('폴더가 제거되었습니다.', 'info');
	}

	function clearAll() {
		folders = [];
		showMessage('모든 폴더가 제거되었습니다.', 'info');
	}

	function showMessage(text: string, type: 'success' | 'error' | 'info') {
		message = { text, type };
		setTimeout(() => {
			message = null;
		}, 3000);
	}
</script>

<div class="container">
	<div class="header">
		<h1>폴더 드래그 & 드롭</h1>
		<button class="btn-select" onclick={selectDirectory}>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M12 4v16m8-8H4"
				/>
			</svg>
			디렉토리 선택
		</button>
	</div>

	<div class="info-box">
		<p>
			<strong>💡 절대 경로 얻기:</strong> 웹 브라우저의 드래그&드롭은 보안상 절대 경로를 제공하지 않습니다.
			<br />
			<strong>"디렉토리 선택"</strong> 버튼을 사용하시면 File System Access API를 통해 실제 디렉토리에 접근할 수 있습니다. (Chrome 86+)
		</p>
	</div>

	<div
		role="region"
		aria-label="폴더 드롭 영역"
		class="drop-zone"
		class:dragging={isDragging}
		ondragenter={handleDragEnter}
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		ondrop={handleDrop}
	>
		<div class="drop-zone-content">
			<svg
				class="drop-icon"
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
				/>
			</svg>
			<p class="drop-text">
				{#if isDragging}
					폴더를 여기에 놓으세요
				{:else}
					폴더를 드래그하여 이곳에 드롭하세요
				{/if}
			</p>
			<p class="drop-hint">폴더만 추가할 수 있습니다 (파일 제외)</p>
		</div>
	</div>

	{#if message}
		<div class="message {message.type}">
			{message.text}
		</div>
	{/if}

	{#if folders.length > 0}
		<div class="folder-list-header">
			<h2>추가된 폴더 ({folders.length})</h2>
			<button class="btn-clear" onclick={clearAll}>모두 제거</button>
		</div>

		<div class="folder-list">
			{#each folders as folder (folder.id)}
				<div class="folder-item">
					<div class="folder-info">
						<svg
							class="folder-icon"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
							/>
						</svg>
						<div class="folder-details">
							<span class="folder-name">{folder.name}</span>
							<span class="folder-path">{folder.path}</span>
						</div>
					</div>
					<button 
						class="btn-remove"
						aria-label="폴더 제거"
						title="폴더 제거"
						onclick={() => removeFolder(folder.id)}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
		background: var(--bg-primary);
		min-height: calc(100vh - 80px);
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
		gap: 1rem;
	}

	h1 {
		color: var(--text-primary);
		margin: 0;
		font-size: 2rem;
		font-weight: 600;
	}

	.btn-select {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1.5rem;
		background: var(--btn-success);
		color: #000;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.btn-select:hover {
		background: var(--btn-success-hover);
		transform: translateY(-2px);
		box-shadow: 0 4px 8px var(--shadow-hover);
	}

	.btn-select svg {
		width: 20px;
		height: 20px;
	}

	.info-box {
		background: var(--status-info);
		border: 1px solid var(--status-info-border);
		border-radius: 8px;
		padding: 1rem 1.5rem;
		margin-bottom: 1.5rem;
	}

	.info-box p {
		margin: 0;
		color: var(--text-primary);
		font-size: 0.9rem;
		line-height: 1.6;
	}

	.info-box strong {
		color: var(--status-info-border);
	}

	.drop-zone {
		border: 3px dashed var(--border-color);
		border-radius: 12px;
		padding: 3rem;
		text-align: center;
		background: var(--bg-secondary);
		transition: all 0.3s ease;
		cursor: pointer;
		margin-bottom: 2rem;
	}

	.drop-zone:hover {
		border-color: var(--btn-primary);
		background: var(--bg-tertiary);
	}

	.drop-zone.dragging {
		border-color: var(--btn-primary);
		background: var(--status-info);
		border-style: solid;
		transform: scale(1.02);
		box-shadow: 0 8px 16px var(--shadow-hover);
	}

	.drop-zone-content {
		pointer-events: none;
	}

	.drop-icon {
		width: 80px;
		height: 80px;
		color: var(--text-secondary);
		margin: 0 auto 1rem;
	}

	.drop-zone.dragging .drop-icon {
		color: var(--btn-primary);
		animation: bounce 0.5s ease infinite;
	}

	@keyframes bounce {
		0%,
		100% {
			transform: translateY(0);
		}
		50% {
			transform: translateY(-10px);
		}
	}

	.drop-text {
		font-size: 1.25rem;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0 0 0.5rem 0;
	}

	.drop-hint {
		font-size: 0.9rem;
		color: var(--text-secondary);
		margin: 0;
	}

	.message {
		padding: 1rem 1.5rem;
		border-radius: 8px;
		margin-bottom: 1.5rem;
		font-weight: 500;
		animation: slideIn 0.3s ease;
	}

	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateY(-10px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.message.success {
		background: var(--status-success);
		color: var(--status-success-border);
		border: 1px solid var(--status-success-border);
	}

	.message.error {
		background: var(--status-error);
		color: var(--status-error-border);
		border: 1px solid var(--status-error-border);
	}

	.message.info {
		background: var(--status-info);
		color: var(--status-info-border);
		border: 1px solid var(--status-info-border);
	}

	.folder-list-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	h2 {
		color: var(--text-primary);
		font-size: 1.5rem;
		font-weight: 600;
		margin: 0;
	}

	.btn-clear {
		padding: 0.5rem 1rem;
		background: var(--btn-secondary);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 0.9rem;
		cursor: pointer;
		transition: all 0.2s ease;
		font-weight: 500;
	}

	.btn-clear:hover {
		background: var(--btn-secondary-hover);
		transform: translateY(-2px);
	}

	.folder-list {
		display: grid;
		gap: 1rem;
	}

	.folder-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.25rem;
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		transition: all 0.2s ease;
	}

	.folder-item:hover {
		background: var(--bg-tertiary);
		box-shadow: 0 4px 8px var(--shadow);
		transform: translateY(-2px);
	}

	.folder-info {
		display: flex;
		align-items: center;
		gap: 1rem;
		flex: 1;
		min-width: 0;
	}

	.folder-icon {
		width: 40px;
		height: 40px;
		color: var(--btn-primary);
		flex-shrink: 0;
	}

	.folder-details {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		min-width: 0;
		flex: 1;
	}

	.folder-name {
		font-size: 1rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.folder-path {
		font-size: 0.85rem;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.btn-remove {
		padding: 0.5rem;
		background: var(--status-error);
		color: var(--status-error-border);
		border: 1px solid var(--status-error-border);
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.btn-remove:hover {
		background: var(--status-error-border);
		color: white;
		transform: scale(1.1);
	}

	.btn-remove svg {
		width: 20px;
		height: 20px;
	}
</style>

