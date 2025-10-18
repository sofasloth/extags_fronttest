<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount, onDestroy } from 'svelte';
  import { basename } from '@tauri-apps/api/path'; // Tauri의 경로 유틸리티 사용
  import { open } from '@tauri-apps/plugin-dialog';
  // import { openFolderIPC, isFolderIPC } from '../types/ipc_command';
  // import { WebView} from "@tauri-apps/api/webview"

	interface FolderItem {
		id: string;
		path: string;
		name: string;
	}

  type TauriDropEvent = {
    paths: string[];
    position: {
      x: number;
      y: number;
    }
  }

	let folders = $state<FolderItem[]>([]);
	let isDragging = $state(false);
	let message = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);
	let unlistenDrop: (() => void) | null = null;
	let unlistenDragEnter: (() => void) | null = null;
	let unlistenDragLeave: (() => void) | null = null;
	
	// 다중 선택
	let selectedFolderIds = $state<Set<string>>(new Set());
	let lastSelectedId = $state<string | null>(null);
	
	// 삭제 확인 모달
	let showDeleteConfirm = $state(false);
	let deleteTargetIds = $state<string[]>([]);
	
	// 경로 충돌 모달
	let showPathConflictModal = $state(false);
	let conflictSupsetFolders = $state<FolderItem[]>([]);
	let conflictSubsetFolders = $state<FolderItem[]>([]);
	let subsetFolderOptions = $state<Map<string, 'replace' | 'keep'>>(new Map()); // key is id
	// let supsetFolderRestriction = $state<Map<string, string[]>>(new Map()); // key is id
	let resolveConflict: ((value: { action: 'ok' | 'cancel', options: Map<string, 'replace' | 'keep'> }) => void) | null = null;
	
	// 페이지네이션
	let currentPage = $state(1);
	const itemsPerPage = 20;
	
	// 페이지네이션된 폴더 목록
	let paginatedFolders = $derived(() => {
		const startIndex = (currentPage - 1) * itemsPerPage;
		const endIndex = startIndex + itemsPerPage;
		return folders.slice(startIndex, endIndex);
	});
	
	let totalPages = $derived(Math.ceil(folders.length / itemsPerPage));

	onMount(async () => {
    console.log("DirectoryManager onMount");
		// Tauri 드래그 앤 드롭 이벤트 리스너
		unlistenDrop = await listen<TauriDropEvent>('tauri://drag-drop', (event) => {
			isDragging = false;
			handleTauriDrop(event.payload.paths);
		});

		unlistenDragEnter = await listen('tauri://drag-enter', () => {
			isDragging = true;
		});

		unlistenDragLeave = await listen('tauri://drag-leave', () => {
			isDragging = false;
		});
	});

	onDestroy(() => {
		// 리스너 정리
		unlistenDrop?.();
		unlistenDragEnter?.();
		unlistenDragLeave?.();
	});

	async function handleTauriDrop(paths: TauriDropEvent['paths']) {
		console.log('=== Tauri Drop Event ===');
		console.log('Received paths:', paths, typeof paths);

		const validFolders: FolderItem[] = [];
		const rejectedFiles: string[] = [];

		// Tauri에서는 절대 경로가 그대로 전달됨
		for (const path of paths) {
			// 폴더인지 파일인지 확인 (간단히 확장자로 판단하거나 백엔드 확인 필요)
      let subsetFolders: FolderItem[] = [];
      let supsetFolders: FolderItem[] = [];

      let folderName: string;
			try {
				folderName = await basename(path);
			} catch (error) {
				console.error('basename 추출 실패:', error);
				// 폴백: 수동 파싱
				folderName = path.replace(/^.*[\\\/]/, '') || 'Unknown';
			}
			
			console.log('Extracted folder name:', folderName);
			const isFolder = await checkIfDirectory(path); 
			
			if (isFolder) {
				// 중복 체크
        if (folders.findIndex(f => f.path === path) !== -1) {
					showMessage(`이미 등록된 폴더입니다: ${folderName}`, "info");
					continue;
				}
        
        // 경로 충돌 확인
        folders.filter(f=> f.path.includes(path) && f.path !== path).forEach(f=> {
          subsetFolders.push(f);
        });

        folders.filter(f=> path.includes(f.path) && f.path !== path).forEach(f=> { 
          supsetFolders.push(f);
        });

				console.log('subsetFolders:', subsetFolders);
				console.log('supsetFolders:', supsetFolders);

				// 충돌이 있으면 다이얼로그 표시
				if (subsetFolders.length > 0 || supsetFolders.length > 0) {
					const result = await showConflictDialog(supsetFolders, subsetFolders);
					
					if (result.action === 'cancel') {
						console.log('사용자가 취소함');
						continue; // 이 경로는 건너뛰기
					}

					// OK를 눌렀을 때 선택한 옵션에 따라 처리
					if (result.action === 'ok') {
						await handleConflictResolution(result.options, supsetFolders, subsetFolders, path);
					}
				}
        
        const newFolderName = path.split(/[/\\]/).pop() || 'Unknown';
				validFolders.push({
					id: `${Date.now()}-${Math.random()}`,
					path: path, // 절대 경로!
					name: newFolderName
				});
			} else {
        showMessage(`선택한 항목이 폴더가 아닙니다: ${path}`, "error");
				rejectedFiles.push(path);
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
				`${rejectedFiles.length}개의 파일은 제외되었습니다.`,
				'error'
			);
		}
	}

	// 폴더인지 확인하는 백엔드 명령 (선택사항)
	async function checkIfDirectory(path: string): Promise<boolean> {
		try {
			// const { invoke } = await import('@tauri-apps/api/core');
			return await isFolderIPC(path);
		} catch(error) {
			console.error("checkIfDirectory error", error);
      return true; // 에러 시 폴더로 가정
		}
	}

	// 충돌 다이얼로그 표시 (Promise 기반)
	function showConflictDialog(
		supsetFolders: FolderItem[], 
		subsetFolders: FolderItem[]
	): Promise<{ action: 'ok' | 'cancel', options: Map<string, 'replace' | 'keep'> }> {
		return new Promise((resolve) => {
			conflictSupsetFolders = supsetFolders;
			conflictSubsetFolders = subsetFolders;
			
			// 각 하위 폴더에 대해 기본값 'replace' 설정
			const options = new Map<string, 'replace' | 'keep'>();
			subsetFolders.forEach(folder => {
				options.set(folder.id, 'replace');
			});
			subsetFolderOptions = options;
			
			resolveConflict = resolve;
			showPathConflictModal = true;
		});
	}

  // 드래그 앤 드롭 이벤트 처리

	// 충돌 모달에서 OK 클릭
	function handleConflictOk() {
		if (resolveConflict) {
			resolveConflict({ 
				action: 'ok', 
				options: subsetFolderOptions 
			});
		}
		closeConflictModal();
	}

	// 충돌 모달에서 Cancel 클릭
	function handleConflictCancel() {
		if (resolveConflict) {
			resolveConflict({ 
				action: 'cancel', 
				options: new Map() 
			});
		}
		closeConflictModal();
	}

	// 충돌 모달 닫기
	function closeConflictModal() {
		showPathConflictModal = false;
		conflictSupsetFolders = [];
		conflictSubsetFolders = [];
		subsetFolderOptions = new Map();
		resolveConflict = null;
	}

	// 충돌 해결 처리
	async function handleConflictResolution(
		options: Map<string, 'replace' | 'keep'>,
		supsetFolders: FolderItem[],
		subsetFolders: FolderItem[],
		newPath: string
	) {
		console.log('충돌 해결 옵션:', options);
		console.log('Supset folders:', supsetFolders);
		console.log('Subset folders:', subsetFolders);
		console.log('New path:', newPath);

		// 상위 폴더는 항상 유지 (divide)
		// 하위 폴더는 각각의 선택에 따라 처리
		const idsToRemove: string[] = [];
		let replaceCount = 0;
		let keepCount = 0;

		subsetFolders.forEach(folder => {
			const option = options.get(folder.id);
			if (option === 'replace') {
				idsToRemove.push(folder.id);
				replaceCount++;
			} else {
				keepCount++;
			}
		});

		// 선택된 폴더들 제거
		if (idsToRemove.length > 0) {
			folders = folders.filter(f => !idsToRemove.includes(f.id));
		}

		// 결과 메시지
		const messages: string[] = [];
		if (supsetFolders.length > 0) {
			messages.push(`상위 폴더 ${supsetFolders.length}개는 유지됩니다`);
		}
		if (replaceCount > 0) {
			messages.push(`하위 폴더 ${replaceCount}개가 교체되었습니다`);
		}
		if (keepCount > 0) {
			messages.push(`하위 폴더 ${keepCount}개는 별도로 유지됩니다`);
		}
		
		if (messages.length > 0) {
			showMessage(messages.join(', '), 'info');
		}
	}

	function handleFolderClick(event: MouseEvent, folderId: string) {
		if (event.ctrlKey || event.metaKey) {
			// Ctrl/Cmd 클릭: 개별 토글
			if (selectedFolderIds.has(folderId)) {
				selectedFolderIds.delete(folderId);
			} else {
				selectedFolderIds.add(folderId);
			}
			selectedFolderIds = new Set(selectedFolderIds);
			lastSelectedId = folderId;
		} else if (event.shiftKey && lastSelectedId) {
			// Shift 클릭: 범위 선택
			const lastIndex = folders.findIndex(f => f.id === lastSelectedId);
			const currentIndex = folders.findIndex(f => f.id === folderId);
			
			if (lastIndex !== -1 && currentIndex !== -1) {
				const start = Math.min(lastIndex, currentIndex);
				const end = Math.max(lastIndex, currentIndex);
				
				for (let i = start; i <= end; i++) {
					selectedFolderIds.add(folders[i].id);
				}
				selectedFolderIds = new Set(selectedFolderIds);
			}
		} else {
			// 일반 클릭: 단일 선택
      if (lastSelectedId === folderId) {
        selectedFolderIds = new Set();
        lastSelectedId = null;
      } else {
        selectedFolderIds = new Set([folderId]);
        lastSelectedId = folderId;
      }
		}
	}

	function clearSelection() {
		selectedFolderIds = new Set();
		lastSelectedId = null;
	}

	function confirmDeleteFolder(id: string) {
		deleteTargetIds = [id];
		showDeleteConfirm = true;
	}

	function confirmDeleteSelected() {
		deleteTargetIds = Array.from(selectedFolderIds);
		showDeleteConfirm = true;
	}

	function executeDelete() {
		folders = folders.filter((f) => !deleteTargetIds.includes(f.id));
		selectedFolderIds = new Set();
		lastSelectedId = null;
		
		// 현재 페이지가 비어있으면 이전 페이지로
		if (paginatedFolders().length === 0 && currentPage > 1) {
			currentPage--;
		}
		
		showMessage(`${deleteTargetIds.length}개의 폴더가 제거되었습니다.`, 'info');
		showDeleteConfirm = false;
		deleteTargetIds = [];
	}

	function cancelDelete() {
		showDeleteConfirm = false;
		deleteTargetIds = [];
	}

	function clearAll() {
		deleteTargetIds = folders.map(f => f.id);
		showDeleteConfirm = true;
	}
	
	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
		}
	}
	
	function nextPage() {
		if (currentPage < totalPages) {
			currentPage++;
		}
	}
	
	function prevPage() {
		if (currentPage > 1) {
			currentPage--;
		}
	}

	function showMessage(text: string, type: 'success' | 'error' | 'info') {
		message = { text, type };
		setTimeout(() => {
			message = null;
		}, 3000);
	}

  async function openFolderorCopyPath(event: Event, path: string) {
    event.stopPropagation();
    console.log(event);
    console.log(path);
    try {
      if((event as KeyboardEvent).ctrlKey) {
        // await openFolderIPC(path);
        await openFolderIPC(path);
      } else {
        await navigator.clipboard.writeText(path);
        showMessage(`경로가 복사되었습니다.${path}`, 'success');
      }
    } catch (error) {
      console.error(error);
    }
  }

  async function openFolderDialog() {
    const folders = await open({
      multiple: true,
      directory: true,
    });
    console.log(folders);
    if (folders) {
      handleTauriDrop(folders);
    }
  }
</script>

<div class="container">
	<h1>Tauri 폴더 드래그 & 드롭</h1>

	<div
		class="drop-zone"
		class:dragging={isDragging}
    onclick={openFolderDialog}
		onkeydown={(event) => {
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault();
				openFolderDialog();
			}
		}}
		ondragover={(e) => {
			e.preventDefault();
			isDragging = true;
		}}
		ondragleave={(e) => {
			e.preventDefault();
			isDragging = false;
		}}
		ondrop={(e) => {
			e.preventDefault();
			isDragging = false;
		}}
		role="button"
		tabindex="0"
	>
		<div class="drop-zone-content">
			<svg class="drop-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
			</svg>
			<p class="drop-text">
				{#if isDragging}
					폴더를 여기에 놓으세요
				{:else}
					폴더를 드래그하여 이곳에 드롭하세요
				{/if}
			</p>
			<p class="drop-hint">Tauri 환경: 절대 경로가 제공됩니다</p>
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
			<div class="header-actions">
				{#if selectedFolderIds.size > 0}
					<button class="btn-action" onclick={clearSelection}>
						선택 해제 ({selectedFolderIds.size})
					</button>
					<button class="btn-action btn-delete-selected" onclick={confirmDeleteSelected}>
						선택 항목 삭제
					</button>
				{/if}
				<button class="btn-clear" onclick={clearAll}>모두 제거</button>
			</div>
		</div>

		<div class="folder-list">
			{#each paginatedFolders() as folder (folder.id)}
				<div 
					class="folder-item" 
					class:selected={selectedFolderIds.has(folder.id)}
					onclick={(event) => handleFolderClick(event, folder.id)}
					onkeydown={(event) => {
						if (event.key === 'Enter' || event.key === ' ') {
							event.preventDefault();
							handleFolderClick(event as any, folder.id);
						}
					}}
					role="button"
					tabindex="0"
				>
					<div class="folder-info">
						<div class="folder-details">
							<span class="folder-name">{folder.name}</span>
							<span 
                class="folder-path" 
                onclick={(event) => openFolderorCopyPath(event, folder.path)}
                onkeydown={(event) => {
									if (event.key === 'Enter' || event.key === ' ') {
										event.preventDefault();
										openFolderorCopyPath(event, folder.path);
									}
								}}
                role="button"
                tabindex="0">
                {folder.path}
              </span>
						</div>
					</div>
					<button 
						class="btn-remove"
						onclick={(event) => {
							event.stopPropagation();
							confirmDeleteFolder(folder.id);
						}}
					>
						✕
					</button>
				</div>
			{/each}
		</div>

		{#if totalPages > 1}
			<div class="pagination">
				<button 
					class="pagination-btn" 
					onclick={prevPage} 
					disabled={currentPage === 1}
				>
					이전
				</button>
				
				<div class="pagination-pages">
					{#each Array(totalPages) as _, i}
						{@const pageNum = i + 1}
						<button 
							class="pagination-page"
							class:active={currentPage === pageNum}
							onclick={() => goToPage(pageNum)}
						>
							{pageNum}
						</button>
					{/each}
				</div>
				
				<button 
					class="pagination-btn" 
					onclick={nextPage}
					disabled={currentPage === totalPages}
				>
					다음
				</button>
			</div>
		{/if}
	{/if}
</div>

<!-- 삭제 확인 모달 -->
{#if showDeleteConfirm}
	<div 
		class="modal-overlay" 
		onclick={cancelDelete}
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				cancelDelete();
			}
		}}
		role="dialog"
		aria-modal="true"
		aria-labelledby="delete-confirm-title"
		tabindex="-1"
	>
		<div 
			class="modal-content" 
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="none"
		>
			<h3 id="delete-confirm-title">삭제 확인</h3>
			<p>
				{#if deleteTargetIds.length === 1}
					선택한 폴더를 삭제하시겠습니까? <br>
          폴더를 삭제하면 해당 경로 내 원본 파일은 삭제가 안 되지만 연관된 태그, 메모 등의 정보는 삭제됩니다.
				{:else}
					선택한 {deleteTargetIds.length}개의 폴더를 삭제하시겠습니까?
          폴더를 삭제하면 해당 경로 내 원본 파일은 삭제가 안 되지만 연관된 태그, 메모 등의 정보는 삭제됩니다.
				{/if}
			</p>
			<div class="modal-actions">
				<button class="btn-modal btn-cancel" onclick={cancelDelete}>
					취소
				</button>
				<button class="btn-modal btn-confirm" onclick={executeDelete}>
					삭제
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- 경로 충돌 해결 모달 -->
{#if showPathConflictModal}
	<div 
		class="modal-overlay" 
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				handleConflictCancel();
			}
		}}
		role="dialog"
		aria-modal="true"
		aria-labelledby="conflict-title"
		tabindex="-1"
	>
		<div 
			class="modal-content conflict-modal" 
			onclick={(e) => e.stopPropagation()}
			role="none"
		>
			<h3 id="conflict-title">경로 충돌 감지</h3>
			
			{#if conflictSupsetFolders.length > 0}
				<div class="conflict-section">
					<h4>ℹ️ 상위 폴더가 이미 등록되어 있습니다</h4>
					<p class="conflict-info-text">상위 폴더는 자동으로 유지됩니다 (Divide)</p>
					<div class="conflict-list">
						{#each conflictSupsetFolders as folder}
							<div class="conflict-item">
								<span class="conflict-folder-name">{folder.name}</span>
								<span class="conflict-folder-path">{folder.path}</span>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			{#if conflictSubsetFolders.length > 0}
				<div class="conflict-section">
					<h4>⚠️ 하위 폴더가 이미 등록되어 있습니다</h4>
					<p class="conflict-info-text">각 폴더별로 처리 방법을 선택하세요:
						<br/>- 교체: 충돌 폴더를 새로운 폴더로 대체합니다.	
						<br/>- 분리: 충돌 폴더를 별도의 폴더로 유지합니다.
					</p>
					<div class="conflict-list-with-options">
						{#each conflictSubsetFolders as folder}
							<div class="conflict-item-with-option">
								<div class="conflict-item-info">
									<span class="conflict-folder-name">{folder.name}</span>
									<span class="conflict-folder-path">{folder.path}</span>
								</div>
								<div class="conflict-item-radio">
									<label class="inline-radio-option">
										<input 
											type="radio" 
											name="conflict-option-{folder.id}" 
											value="replace"
											checked={subsetFolderOptions.get(folder.id) === 'replace'}
											onchange={() => {
												subsetFolderOptions.set(folder.id, 'replace');
												subsetFolderOptions = new Map(subsetFolderOptions);
											}}
										/>
										<span class="radio-label">교체</span>
									</label>
									<label class="inline-radio-option">
										<input 
											type="radio" 
											name="conflict-option-{folder.id}" 
											value="keep"
											checked={subsetFolderOptions.get(folder.id) === 'keep'}
											onchange={() => {
												subsetFolderOptions.set(folder.id, 'keep');
												subsetFolderOptions = new Map(subsetFolderOptions);
											}}
										/>
										<span class="radio-label">분리</span>
									</label>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<div class="modal-actions">
				<button class="btn-modal btn-cancel" onclick={handleConflictCancel}>
					Cancel
				</button>
				<button class="btn-modal btn-confirm" onclick={handleConflictOk}>
					OK
				</button>
			</div>
		</div>
	</div>
{/if}

<style>

  *{
    user-select: none;
  }

	.container {
		max-width: 1080px;
		width: 100%;
		max-height: calc(min(90vh, 1200px));
		height: fit-content;
		padding: 2rem;
		background: var(--bg-primary);
		margin: 2rem 4rem;
		overflow: hidden;
		border-radius: 12px;
		box-shadow: 0 4px 24px var(--shadow);
		z-index: 101;
		
		display: flex;
		flex-direction: column;
	}

	h1 {
		color: var(--text-primary);
		margin-bottom: 2rem;
		font-size: 2rem;
		font-weight: 600;
		flex-shrink: 0;
	}

	.drop-zone {
		position: sticky;
		top: 0;
		z-index: 10;
		border: 3px dashed var(--border-color);
		border-radius: 12px;
		padding: 3rem;
		text-align: center;
		background: var(--bg-secondary);
		transition: all 0.3s ease;
		cursor: pointer;
		margin-bottom: 2rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		flex-shrink: 0;
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
    pointer-events: all;
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
		flex-shrink: 0;
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
		gap: 1rem;
		flex-shrink: 0;
	}

	h2 {
		color: var(--text-primary);
		font-size: 1.5rem;
		font-weight: 600;
		margin: 0;
	}

	.header-actions {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.btn-action {
		padding: 0.5rem 1rem;
		background: var(--btn-primary);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 0.9rem;
		cursor: pointer;
		transition: all 0.2s ease;
		font-weight: 500;
		white-space: nowrap;
	}

	.btn-action:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 8px var(--shadow);
	}

	.btn-delete-selected {
		background: var(--btn-secondary);
	}

	.btn-delete-selected:hover {
		background: var(--btn-secondary-hover);
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
		position: relative;
		display: grid;
		gap: 1rem;
		overflow-y: auto;
		padding-right: 0.5rem;
		flex: 1;
		min-height: 0;

		align-content: start;
		grid-auto-rows:min-content;
	}
	
	.folder-list::-webkit-scrollbar {
		width: 8px;
	}
	
	.folder-list::-webkit-scrollbar-track {
		background: var(--bg-secondary);
		border-radius: 4px;
	}
	
	.folder-list::-webkit-scrollbar-thumb {
		background: var(--border-color);
		border-radius: 4px;
	}
	
	.folder-list::-webkit-scrollbar-thumb:hover {
		background: var(--btn-primary);
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
		cursor: pointer;
	}

	.folder-item:hover {
		background: var(--bg-tertiary);
		box-shadow: 0 4px 8px var(--shadow);
		transform: translateY(-2px);
	}

	.folder-item.selected {
		background: var(--status-info);
		border-color: var(--btn-primary);
		border-width: 2px;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
	}

	.folder-item.selected:hover {
		background: var(--status-info);
		border-color: var(--btn-primary);
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
    width: fit-content;
		font-size: 0.85rem;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;

    cursor: pointer;
    transition: all 0.2s ease;
	}

  .folder-path:hover {
    color: var(--text-primary);
    text-decoration: underline;
    text-decoration-color: var(--text-secondary);
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
  }

  .folder-path:active {
    color: var(--text-secondary);
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

	.pagination {
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 1rem;
		margin-top: 2rem;
		padding: 1rem;
		background: var(--bg-secondary);
		border-radius: 8px;
		flex-shrink: 0;
	}

	.pagination-btn {
		padding: 0.5rem 1rem;
		background: var(--btn-primary);
		color: white;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.2s ease;
		font-weight: 500;
	}

	.pagination-btn:hover:not(:disabled) {
		background: var(--btn-primary-hover);
		transform: translateY(-2px);
	}

	.pagination-btn:disabled {
		background: var(--border-color);
		cursor: not-allowed;
		opacity: 0.5;
	}

	.pagination-pages {
		display: flex;
		gap: 0.5rem;
		max-width: 500px;
		overflow-x: auto;
		padding: 0.25rem;
	}

	.pagination-pages::-webkit-scrollbar {
		height: 4px;
	}

	.pagination-pages::-webkit-scrollbar-track {
		background: var(--bg-tertiary);
		border-radius: 2px;
	}

	.pagination-pages::-webkit-scrollbar-thumb {
		background: var(--border-color);
		border-radius: 2px;
	}

	.pagination-page {
		min-width: 40px;
		padding: 0.5rem;
		background: var(--bg-tertiary);
		color: var(--text-primary);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.2s ease;
		font-weight: 500;
	}

	.pagination-page:hover {
		background: var(--bg-secondary);
		border-color: var(--btn-primary);
	}

	.pagination-page.active {
		background: var(--btn-primary);
		color: white;
		border-color: var(--btn-primary);
	}

	/* 모달 스타일 */
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
		animation: fadeIn 0.2s ease;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	.modal-content {
		background: var(--bg-secondary);
		border-radius: 12px;
		padding: 2rem;
		max-width: 500px;
		width: 90%;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
		animation: slideUp 0.3s ease;
	}

	@keyframes slideUp {
		from {
			transform: translateY(20px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	.modal-content h3 {
		color: var(--text-primary);
		font-size: 1.5rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
	}

	.modal-content p {
		color: var(--text-secondary);
		font-size: 1rem;
		line-height: 1.5;
		margin: 0 0 1.5rem 0;
	}

	.modal-actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
	}

	.btn-modal {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.btn-cancel {
		background: var(--bg-tertiary);
		color: var(--text-primary);
		border: 1px solid var(--border-color);
	}

	.btn-cancel:hover {
		background: var(--bg-primary);
		transform: translateY(-2px);
	}

	.btn-confirm {
		background: var(--btn-secondary);
		color: white;
	}

	.btn-confirm:hover {
		background: var(--btn-secondary-hover);
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
	}

	.btn-confirm:disabled {
		background: var(--border-color);
		cursor: not-allowed;
		opacity: 0.5;
	}

	/* 충돌 모달 스타일 */
	.conflict-modal {
		max-width: 600px;
		max-height: 80vh;
		overflow-y: auto;
	}

	.conflict-section {
		margin-bottom: 1.5rem;
		padding: 1rem;
		background: var(--bg-tertiary);
		border-radius: 8px;
		border: 1px solid var(--border-color);
	}

	.conflict-section h4 {
		color: var(--text-primary);
		font-size: 1rem;
		font-weight: 600;
		margin: 0 0 0.75rem 0;
	}

	.conflict-info-text {
		color: var(--text-secondary);
		font-size: 0.9rem;
		margin: 0 0 0.75rem 0;
		line-height: 1.4;
	}

	.conflict-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.conflict-list-with-options {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.conflict-item {
		display: flex;
		flex-direction: column;
		padding: 0.75rem;
		background: var(--bg-primary);
		border-radius: 6px;
		border: 1px solid var(--border-color);
	}

	.conflict-item-with-option {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		padding: 1rem;
		background: var(--bg-primary);
		border-radius: 8px;
		border: 1px solid var(--border-color);
	}

	.conflict-item-info {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.conflict-item-radio {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.inline-radio-option {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		padding: 0.5rem 1rem;
		background: var(--bg-tertiary);
		border: 2px solid var(--border-color);
		border-radius: 6px;
		transition: all 0.2s ease;
		flex: 1;
	}

	.inline-radio-option:hover {
		background: var(--bg-secondary);
		border-color: var(--btn-primary);
	}

	.inline-radio-option:has(input[type="radio"]:checked) {
		border-color: var(--btn-primary);
		background: var(--status-info);
	}

	.inline-radio-option input[type="radio"] {
		margin: 0;
		cursor: pointer;
	}

	.radio-label {
		font-weight: 500;
		color: var(--text-primary);
		font-size: 0.95rem;
		user-select: none;
	}

	.conflict-folder-name {
		font-weight: 600;
		color: var(--text-primary);
		font-size: 0.95rem;
		margin-bottom: 0.25rem;
	}

	.conflict-folder-path {
		font-size: 0.85rem;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}


	.conflict-modal::-webkit-scrollbar {
		width: 8px;
	}

	.conflict-modal::-webkit-scrollbar-track {
		background: var(--bg-secondary);
		border-radius: 4px;
	}

	.conflict-modal::-webkit-scrollbar-thumb {
		background: var(--border-color);
		border-radius: 4px;
	}

	.conflict-modal::-webkit-scrollbar-thumb:hover {
		background: var(--btn-primary);
	}
</style>