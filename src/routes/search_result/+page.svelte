<script lang="ts">
	type FileItem = {
		id: number;
		fileName: string;
		modified: string;
		path: string;
		size: string;
		sizeBytes: number;
		tags: string[];
	};

	// Mock data - 100개의 파일 항목 생성
	let files = $state<FileItem[]>(
		Array.from({ length: 100 }, (_, i) => ({
			id: i + 1,
			fileName: `Excel file ${i + 1}.xlsx`,
			modified: '25.01.01 09:00',
			path: `C:\\(중략)\\Users\\username\\Documents\\Excel file${i + 1}.xlsx`,
			size: '831.2 kB',
			sizeBytes: 831200,
			tags: ['TAG 1', 'TAG 2']
		}))
	);

	// 칼럼 너비 상태
	let columnWidths = $state({
		index: 60,
		fileName: 250,
		modified: 150,
		path: 350,
		size: 120,
		tag: 250
	});

	// 리사이징 상태
	let resizingColumn: string | null = null;
	let resizeStartX = 0;
	let resizeStartWidth = 0;

	// 통계 계산
	let totalFiles = $derived(files.length);
	let totalSize = $derived(
		files.reduce((sum, file) => sum + file.sizeBytes, 0)
	);
	let totalSizeMB = $derived((totalSize / (1024 * 1024)).toFixed(1));
	let usagePercentage = $derived(15.2); // 예시 값

	function startResize(column: string, e: MouseEvent) {
		e.preventDefault();
		resizingColumn = column;
		resizeStartX = e.clientX;
		resizeStartWidth = columnWidths[column as keyof typeof columnWidths];
	}

	function handleMouseMove(e: MouseEvent) {
		if (resizingColumn) {
			const delta = e.clientX - resizeStartX;
			const newWidth = Math.max(50, resizeStartWidth + delta);
			columnWidths = {
				...columnWidths,
				[resizingColumn]: newWidth
			};
		}
	}

	function handleMouseUp() {
		resizingColumn = null;
	}

	// 경로를 앞에서부터 생략하여 표시
	function truncatePathFromStart(path: string, maxLength: number): string {
		if (path.length <= maxLength) return path;
		return '...' + path.slice(-(maxLength - 3));
	}

	// 파일 아이콘 선택
	function getFileIcon(fileName: string): string {
		if (fileName.endsWith('.xlsx') || fileName.endsWith('.xls')) return '📊';
		if (fileName.endsWith('.pdf')) return '📄';
		if (fileName.endsWith('.jpg') || fileName.endsWith('.png')) return '🖼️';
		return '📁';
	}
</script>

<svelte:window onmousemove={handleMouseMove} onmouseup={handleMouseUp} />

<div class="file-explorer">
	<!-- 제목 표시줄 -->
	<div class="title-bar">
		<div class="title-info">
			<span class="tag-display">
				<span class="tag-badge active">🎨 TAG NAME 1</span>
				<button class="tag-close">✕</button>
			</span>
			<span class="divider">/</span>
			<span class="tag-display">
				<span class="tag-badge">🎨 TAG 2</span>
				<button class="tag-close">✕</button>
			</span>
		</div>
		<div class="title-stats">
			<span>{totalFiles} files</span>
			<span>·</span>
			<span>{totalSizeMB} MB</span>
			<span>·</span>
			<span>{usagePercentage}%</span>
		</div>
	</div>

	<!-- 칼럼 헤더 -->
	<div class="column-header">
		<div class="column column-index" style="width: {columnWidths.index}px;">
			<span>i</span>
		</div>
		<div class="column column-resizable" style="width: {columnWidths.fileName}px;">
			<span>file_name</span>
			<button class="resize-handle" aria-label="Resize column" onmousedown={(e) => startResize('fileName', e)}></button>
		</div>
		<div class="column column-resizable" style="width: {columnWidths.modified}px;">
			<span>modified</span>
			<button class="resize-handle" aria-label="Resize column" onmousedown={(e) => startResize('modified', e)}></button>
		</div>
		<div class="column column-resizable" style="width: {columnWidths.path}px;">
			<span>path</span>
			<button class="resize-handle" aria-label="Resize column" onmousedown={(e) => startResize('path', e)}></button>
		</div>
		<div class="column column-resizable" style="width: {columnWidths.size}px;">
			<span>size</span>
			<button class="resize-handle" aria-label="Resize column" onmousedown={(e) => startResize('size', e)}></button>
		</div>
		<div class="column column-resizable" style="width: {columnWidths.tag}px;">
			<span>tag</span>
			<button class="resize-handle" aria-label="Resize column" onmousedown={(e) => startResize('tag', e)}></button>
		</div>
	</div>

	<!-- 파일 목록 -->
	<div class="file-list">
		{#each files as file (file.id)}
			<div class="file-row">
				<div class="file-cell cell-index" style="width: {columnWidths.index}px;">
					<span class="index-number">{file.id}</span>
				</div>
				<div class="file-cell cell-filename" style="width: {columnWidths.fileName}px;">
					<span class="file-icon">{getFileIcon(file.fileName)}</span>
					<span class="file-name" title={file.fileName}>{file.fileName}</span>
				</div>
				<div class="file-cell" style="width: {columnWidths.modified}px;">
					<span class="cell-badge">{file.modified}</span>
				</div>
				<div class="file-cell cell-path" style="width: {columnWidths.path}px;">
					<span class="path-text" title={file.path}>{file.path}</span>
				</div>
				<div class="file-cell" style="width: {columnWidths.size}px;">
					<span class="size-badge">{file.size}</span>
				</div>
				<div class="file-cell cell-tags" style="width: {columnWidths.tag}px;">
					{#each file.tags as tag}
						<span class="item-tag">
							<span class="tag-icon">🎨</span>
							<span class="tag-name">{tag}</span>
							<button class="tag-remove">✕</button>
						</span>
					{/each}
				</div>
			</div>
		{/each}
	</div>

	<!-- 스크롤바 인디케이터 (커스텀 스타일링을 위해) -->
	<div class="scrollbar-indicator"></div>
</div>

<style>
	.file-explorer {
		width: 100%;
		height: 100vh;
		display: flex;
		flex-direction: column;
		background: var(--bg-primary);
		color: var(--text-primary);
		overflow: hidden;
	}

	/* 제목 표시줄 */
	.title-bar {
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
		font-size: 12px;
	}

	.title-info {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.tag-display {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.tag-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: var(--bg-tertiary);
		border-radius: 4px;
		font-size: 11px;
		color: var(--text-primary);
	}

	.tag-badge.active {
		background: var(--btn-primary);
		color: white;
	}

	.tag-close {
		background: none;
		border: none;
		color: var(--text-muted);
		font-size: 14px;
		cursor: pointer;
		padding: 0 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 2px;
		transition: all 0.15s ease;
	}

	.tag-close:hover {
		background: rgba(255, 85, 85, 0.2);
		color: #ff5555;
	}

	.divider {
		color: var(--text-muted);
		font-size: 12px;
	}

	.title-stats {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		color: var(--text-secondary);
	}

	/* 칼럼 헤더 */
	.column-header {
		display: flex;
		height: 36px;
		background: var(--bg-secondary);
		border-bottom: 2px solid var(--border-color);
		flex-shrink: 0;
		user-select: none;
	}

	.column {
		display: flex;
		align-items: center;
		padding: 0 12px;
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		border-right: 1px solid var(--border-color);
		position: relative;
		flex-shrink: 0;
	}

	.column-index {
		justify-content: center;
		background: var(--bg-tertiary);
		font-weight: 700;
	}

	.column-resizable {
		position: relative;
	}

	.resize-handle {
		position: absolute;
		right: 0;
		top: 0;
		bottom: 0;
		width: 8px;
		cursor: col-resize;
		z-index: 10;
		transition: background-color 0.15s ease;
		border: none;
		background: transparent;
		padding: 0;
	}

	.resize-handle:hover {
		background-color: var(--btn-primary);
	}

	/* 파일 목록 */
	.file-list {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		background: var(--bg-primary);
	}

	.file-list::-webkit-scrollbar {
		width: 14px;
	}

	.file-list::-webkit-scrollbar-track {
		background: var(--bg-secondary);
		border-left: 1px solid var(--border-color);
	}

	.file-list::-webkit-scrollbar-thumb {
		background: var(--bg-tertiary);
		border: 3px solid var(--bg-secondary);
		border-radius: 8px;
	}

	.file-list::-webkit-scrollbar-thumb:hover {
		background: var(--text-muted);
	}

	.file-row {
		display: flex;
		height: fit-content;
		/* border-bottom: 1px solid var(--border-color); */
		transition: background-color 0.1s ease;
		cursor: pointer;
	}

	.file-row:hover {
		background: var(--bg-secondary);
	}

	.file-cell {
		display: flex;
		align-items: center;
		padding: 0 12px;
		font-size: 13px;
		border-right: 1px solid var(--border-color);
		overflow: hidden;
		flex-shrink: 0;
		color: var(--text-primary);
	}

	.cell-index {
		justify-content: center;
		background: var(--bg-secondary);
		font-weight: 600;
		color: var(--text-muted);
	}

	.index-number {
		font-size: 12px;
		font-family: 'Courier New', monospace;
	}

	.cell-filename {
		gap: 8px;
	}

	.file-icon {
		font-size: 18px;
		flex-shrink: 0;
	}

	.file-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.cell-badge {
		display: inline-flex;
		align-items: center;
		padding: 4px 10px;
		background: var(--bg-tertiary);
		border-radius: 4px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.cell-path {
		color: var(--text-secondary);
		direction: rtl;
		text-align: left;
	}

	.path-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: ltr;
		unicode-bidi: plaintext;
		display: block;
		width: 100%;
	}

	.size-badge {
		display: inline-flex;
		align-items: center;
		padding: 4px 10px;
		background: var(--bg-tertiary);
		border-radius: 4px;
		font-size: 12px;
		color: var(--text-secondary);
		font-family: 'Courier New', monospace;
	}

	.cell-tags {
		gap: 6px;
		overflow-x: auto;
		overflow-y: hidden;
	}

	.cell-tags::-webkit-scrollbar {
		height: 4px;
	}

	.cell-tags::-webkit-scrollbar-track {
		background: transparent;
	}

	.cell-tags::-webkit-scrollbar-thumb {
		background: var(--border-color);
		border-radius: 2px;
	}

	.item-tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 12px;
		font-size: 11px;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.tag-icon {
		font-size: 12px;
	}

	.tag-name {
		font-weight: 500;
	}

	.tag-remove {
		background: none;
		border: none;
		color: var(--text-muted);
		font-size: 12px;
		cursor: pointer;
		padding: 0;
		width: 14px;
		height: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		transition: all 0.15s ease;
	}

	.tag-remove:hover {
		background: rgba(255, 85, 85, 0.2);
		color: #ff5555;
	}

	/* 스크롤바 인디케이터 */
	.scrollbar-indicator {
		position: absolute;
		right: 0;
		top: 68px;
		bottom: 0;
		width: 1px;
		background: var(--border-color);
		pointer-events: none;
	}

	/* 다크 모드 조정 */
	:global(.dark) .file-row:hover {
		background: rgba(255, 255, 255, 0.05);
	}

	:global(.dark) .cell-badge,
	:global(.dark) .size-badge {
		background: var(--bg-tertiary);
		color: var(--text-secondary);
	}
</style>

