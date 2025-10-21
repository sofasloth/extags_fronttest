<script lang="ts">
	import { onMount } from 'svelte';
	import {slide} from 'svelte/transition';

	type Memo = {
		id: number;
		title: string;
		content: string;
		timestamp: string;
	};

	type LinkedFile = {
		id: number;
		name: string;
		path: string;
		size: string;
		date: string;
		tags: string[];
	};

	type SectionVisibility = {
		id: string;
		name: string;
		visible: boolean;
		position: 'above' | 'below' | 'visible';
	};

	let fileName = $state('Excel file 1.xlsx');
	let fileSize = $state('10 MB');
	let filePath = $state('C:\\...\\(중략)/Users/username/Documents/Excel file1.xlsx');
	let created = $state('25.01.01 09:00');
	let modified = $state('25.01.01 09:00');
	let fileTypeTags = $state(['엑셀', '.xlsx', '경로', '양식']);
	
	let tags = $state([
		{ id: 1, name: 'TAG 1', icon: '❄️' },
		{ id: 2, name: 'TAG 2', icon: '❄️' },
		{ id: 3, name: 'TAG NAME 3', icon: '❄️' },
		{ id: 4, name: 'TAG 4', icon: '❄️' }
	]);

	let memos = $state<Memo[]>([
		{
			id: 1,
			title: '새 메모',
			content: '아무 내용을 채워주세요.',
			timestamp: '2025-10-08 17:11 PM'
		},
		{
			id: 2,
			title: '대체 디자인',
			content: '아무 내용이나 채워주세요...',
			timestamp: '2025-10-08 17:11 PM'
		}
	]);

	let linkedFiles = $state<LinkedFile[]>([
		{
			id: 1,
			name: 'Excel file 1.xlsx',
			path: 'C:\\...../username/Documents/Excel file1.xlsx',
			size: '831.2 kB',
			date: '25.01.01 09:00',
			tags: ['TAG 2']
		},
		{
			id: 2,
			name: 'Excel file 1.xlsx',
			path: 'C:\\...../username/Documents/Excel file1.xlsx',
			size: '831.2 kB',
			date: '25.01.01 09:00',
			tags: ['TAG 2']
		},
		{
			id: 3,
			name: 'Excel file 1.xlsx',
			path: 'C:\\...../username/Documents/Excel file1.xlsx',
			size: '831.2 kB',
			date: '25.01.01 09:00',
			tags: ['TAG 2']
		}
	]);

	let showNewMemoDialog = $state(false);
	let newMemoTitle = $state('');
	let newMemoContent = $state('');

	// Fold/Unfold states
	let foldedSections = $state({
		header: false,
		preview: false,
		fileInfo: false,
		tags: false,
		memo: false,
		linked: false
	});

	// Section visibility tracking
	let sectionVisibility = $state<SectionVisibility[]>([]);
	let panelContentRef: HTMLElement | null = null;
	let sectionRefs: { [key: string]: HTMLElement | null } = {};

	function removeTag(tagId: number) {
		tags = tags.filter(t => t.id !== tagId);
	}

	function addMemo() {
		showNewMemoDialog = true;
	}

	function saveMemo() {
		if (newMemoTitle.trim() || newMemoContent.trim()) {
			const now = new Date();
			const timestamp = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${now.getHours()}:${String(now.getMinutes()).padStart(2, '0')} ${now.getHours() >= 12 ? 'PM' : 'AM'}`;
			
			memos = [...memos, {
				id: memos.length + 1,
				title: newMemoTitle || '새 메모',
				content: newMemoContent || '내용 없음',
				timestamp
			}];
			
			newMemoTitle = '';
			newMemoContent = '';
			showNewMemoDialog = false;
		}
	}

	function cancelMemo() {
		newMemoTitle = '';
		newMemoContent = '';
		showNewMemoDialog = false;
	}

	function editMemo(memoId: number) {
		console.log('Edit memo:', memoId);
	}

	function deleteMemo(memoId: number) {
		memos = memos.filter(m => m.id !== memoId);
	}

	function toggleSection(section: keyof typeof foldedSections) {
		foldedSections[section] = !foldedSections[section];
	}

	function scrollToSection(sectionId: string) {
		const element = sectionRefs[sectionId];
		if (element && panelContentRef) {
			const offsetTop = element.offsetTop - panelContentRef.offsetTop;
			panelContentRef.scrollTo({
				top: offsetTop - 20,
				behavior: 'smooth'
			});
		}
	}

	function checkSectionVisibility() {
		if (!panelContentRef) return;

		const containerRect = panelContentRef.getBoundingClientRect();
		const sections = [
			{ id: 'header', name: '파일 헤더' },
			{ id: 'preview', name: 'Preview' },
			{ id: 'fileInfo', name: '파일 기본 정보' },
			{ id: 'tags', name: 'Tags' },
			{ id: 'memo', name: 'Memo' },
			{ id: 'linked', name: 'Linked' }
		];

		const newVisibility: SectionVisibility[] = [];

		sections.forEach(section => {
			const element = sectionRefs[section.id];
			if (element) {
				const rect = element.getBoundingClientRect();
				const containerTop = containerRect.top;
				const containerBottom = containerRect.bottom;
				
				let position: 'above' | 'below' | 'visible' = 'visible';
				let visible = true;

				if (rect.bottom < containerTop) {
					position = 'above';
					visible = false;
				} else if (rect.top > containerBottom) {
					position = 'below';
					visible = false;
				}

				newVisibility.push({
					id: section.id,
					name: section.name,
					visible,
					position
				});
			}
		});

		sectionVisibility = newVisibility;
	}

	onMount(() => {
		if (panelContentRef) {
			panelContentRef.addEventListener('scroll', checkSectionVisibility);
			// Initial check
			setTimeout(checkSectionVisibility, 100);
		}

		return () => {
			if (panelContentRef) {
				panelContentRef.removeEventListener('scroll', checkSectionVisibility);
			}
		};
	});

	$effect(() => {
		// Recheck visibility when sections are folded/unfolded
		if (Object.values(foldedSections).some(v => v !== undefined)) {
			setTimeout(checkSectionVisibility, 100);
		}
	});
</script>

<div class="panel-container">
	<!-- File Header -->
	<div class="file-header" bind:this={sectionRefs.header}>
		<div class="file-title">
			<button class="fold-btn" onclick={() => toggleSection('header')} title="접기/펼치기">
				{foldedSections.header ? '▸' : '▾'}
			</button>
			<span class="file-icon">❌</span>
			<span class="file-name">{fileName}</span>
		</div>
		<span class="file-size">{fileSize}</span>
	</div>

	<div class="panel-content" bind:this={panelContentRef}>
		<!-- Preview Section -->
		<section class="section" bind:this={sectionRefs.preview}>
			<div 
				class="section-label clickable" 
				role="button"
				tabindex="0"
				onclick={() => toggleSection('preview')}
				onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('preview')}
			>
				<button class="fold-btn-inline" title="접기/펼치기">
					{foldedSections.preview ? '▸' : '▾'}
				</button>
				Preview
			</div>
			{#if !foldedSections.preview}
				<div class="preview-box" in:slide={{duration: 200}} out:slide={{duration: 200}}>
					<div class="preview-text">Preview / Summary</div>
				</div>
			{/if}
		</section>

		<!-- File Info Section -->
		<section class="section" bind:this={sectionRefs.fileInfo}>
			<div 
				class="section-label clickable" 
				role="button"
				tabindex="0"
				onclick={() => toggleSection('fileInfo')}
				onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('fileInfo')}
			>
				<button class="fold-btn-inline" title="접기/펼치기">
					{foldedSections.fileInfo ? '▸' : '▾'}
				</button>
				파일 기본 정보
			</div>
			{#if !foldedSections.fileInfo}
				<div class="file-info" in:slide={{duration: 200}} out:slide={{duration: 200}}>
					<div class="info-row">
						<span class="info-icon">📁</span>
						<span class="info-text">{filePath}</span>
					</div>
					<div class="info-row">
						<span class="info-label">• created :</span>
						<span class="info-value">{created}</span>
						<span class="info-label">• 최근 유입 횟수 : 3회</span>
					</div>
					<div class="info-row">
						<span class="info-label">• modified :</span>
						<span class="info-value">{modified}</span>
						<div class="file-type-tags">
							{#each fileTypeTags as tag}
								<span class="file-type-tag">{tag}</span>
							{/each}
						</div>
					</div>
				</div>
			{/if}
		</section>

		<!-- Tags Section -->
		<section class="section" bind:this={sectionRefs.tags}>
			<div 
				class="section-header clickable" 
				role="button"
				tabindex="0"
				onclick={() => toggleSection('tags')}
				onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('tags')}
			>
				<button class="fold-btn-inline" title="접기/펼치기">
					{foldedSections.tags ? '▸' : '▾'}
				</button>
				<span class="section-icon">❄️</span>
				<span class="section-count">4 Tags</span>
				<div class="section-nav">
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>◂</button>
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>▸</button>
				</div>
			</div>
			{#if !foldedSections.tags}
				<div class="tags-container" in:slide={{duration: 200}} out:slide={{duration: 200}}>
				{#each tags as tag}
					<div class="tag-item">
						<span class="tag-icon">{tag.icon}</span>
						<span class="tag-name">{tag.name}</span>
						<button class="tag-remove" onclick={() => removeTag(tag.id)}>✕</button>
					</div>
				{/each}
			</div>
			{/if}
		</section>

		<!-- Memo Section -->
		<section class="section" bind:this={sectionRefs.memo}>
			<div 
				class="section-header clickable" 
				role="button"
				tabindex="0"
				onclick={() => toggleSection('memo')}
				onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('memo')}
			>
				<button class="fold-btn-inline" title="접기/펼치기">
					{foldedSections.memo ? '▸' : '▾'}
				</button>
				<span>MEMO</span>
				<button class="add-btn" onclick={(e)=>{e.stopPropagation(); addMemo()}}>+</button>
				<button class="expand-btn" onclick={(e)=>{e.stopPropagation();}}>≡</button>
				<span class="section-count">2 Memo</span>
				<div class="section-nav">
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>◂</button>
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>▸</button>
				</div>
			</div>
			{#if !foldedSections.memo}
				<div class="memos-container" in:slide={{duration: 200}} out:slide={{duration: 200}}>
				{#each memos as memo}
					<div class="memo-card">
						<div class="memo-header">
							<h4 class="memo-title">{memo.title}</h4>
							<div class="memo-actions">
								<button class="memo-action-btn expand" title="확장">▭</button>
								<button class="memo-action-btn edit" onclick={() => editMemo(memo.id)} title="수정">📝</button>
								<button class="memo-action-btn delete" onclick={() => deleteMemo(memo.id)} title="삭제">✕</button>
							</div>
						</div>
						<div class="memo-content">
							<p>{memo.content}</p>
						</div>
						<div class="memo-footer">
							<div class="memo-tools">
								<span class="memo-tool">#</span>
								<span class="memo-tool">@</span>
								<span class="memo-tool">😊</span>
							</div>
							<span class="memo-timestamp">{memo.timestamp}</span>
						</div>
					</div>
				{/each}
			</div>
			{/if}

			<!-- New Memo Dialog -->
			{#if showNewMemoDialog}
				<div class="memo-dialog">
					<div class="memo-dialog-header">
						<h4>새 메모</h4>
						<button class="memo-dialog-close" onclick={cancelMemo}>✕</button>
					</div>
					<input
						type="text"
						class="memo-dialog-title"
						placeholder="제목"
						bind:value={newMemoTitle}
					/>
					<textarea
						class="memo-dialog-content"
						placeholder="내용을 입력하세요..."
						bind:value={newMemoContent}
					></textarea>
					<div class="memo-dialog-footer">
						<button class="memo-dialog-btn cancel" onclick={cancelMemo}>취소</button>
						<button class="memo-dialog-btn save" onclick={saveMemo}>저장</button>
					</div>
				</div>
			{/if}
		</section>

		<!-- Linked Section -->
		<section class="section" bind:this={sectionRefs.linked}>
			<div 
				class="section-header clickable" 
				role="button"
				tabindex="0"
				onclick={() => toggleSection('linked')}
				onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleSection('linked')}
			>
				<button class="fold-btn-inline" title="접기/펼치기">
					{foldedSections.linked ? '▸' : '▾'}
				</button>
				<span>Linked</span>
				<button class="expand-btn" onclick={(e)=>{e.stopPropagation();}}>≡</button>
				<span class="section-count">3 Links</span>
				<div class="section-nav">
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>◂</button>
					<button class="nav-btn" onclick={(e)=>{e.stopPropagation();}}>▸</button>
				</div>
			</div>
			{#if !foldedSections.linked}
				<div class="linked-container" in:slide={{duration: 200}} out:slide={{duration: 200}}>
				{#each linkedFiles as file}
					<div class="linked-item">
						<div class="linked-icon">❌</div>
						<div class="linked-info">
							<div class="linked-name-row">
								<span class="linked-name">{file.name}</span>
								<span class="linked-date">{file.date}</span>
								{#each file.tags as tag}
									<span class="linked-tag">{tag}</span>
								{/each}
								<button class="linked-remove">✕</button>
							</div>
							<div class="linked-path-row">
								<span class="linked-path">{file.path}</span>
								<span class="linked-size">{file.size}</span>
							</div>
						</div>
					</div>
				{/each}
			</div>
			{/if}
		</section>
	</div>

	<!-- Top Navigation Bar (for sections below) -->
	{#if sectionVisibility.some(s => s.position === 'below')}
		<div class="nav-bar nav-bar-bottom">
			<span class="nav-bar-label">아래 섹션:</span>
			{#each sectionVisibility.filter(s => s.position === 'below') as section}
				<button class="nav-bar-btn" onclick={() => scrollToSection(section.id)}>
					{section.name}
				</button>
			{/each}
		</div>
	{/if}

	<!-- Bottom Navigation Bar (for sections above) -->
	{#if sectionVisibility.some(s => s.position === 'above')}
		<div class="nav-bar nav-bar-top">
			<span class="nav-bar-label">위 섹션:</span>
			{#each sectionVisibility.filter(s => s.position === 'above') as section}
				<button class="nav-bar-btn" onclick={() => scrollToSection(section.id)}>
					{section.name}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.panel-container {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background: var(--bg-secondary);
		color: var(--text-primary);
		overflow: hidden;
		position: relative;
	}

	.file-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		background: var(--bg-tertiary);
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
	}

	.file-title {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.file-icon {
		font-size: 20px;
	}

	.file-name {
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.file-size {
		font-size: 14px;
		color: var(--text-muted);
		background: var(--bg-secondary);
		padding: 4px 12px;
		border-radius: 12px;
	}

	.panel-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	.panel-content::-webkit-scrollbar {
		width: 8px;
	}

	.panel-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.panel-content::-webkit-scrollbar-thumb {
		background-color: var(--border-color);
		border-radius: 4px;
	}

	.section {
		margin-bottom: 24px;
	}

	.section-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-muted);
		margin-bottom: 8px;
		text-transform: uppercase;
		display: flex;
		align-items: center;
		gap: 6px;
		user-select: none;
	}

	.section-label.clickable {
		cursor: pointer;
		padding: 4px 8px;
		margin-left: -8px;
		border-radius: 4px;
		transition: all 0.2s ease;
	}

	.section-label.clickable:hover {
		background: var(--bg-tertiary);
	}

	.fold-btn, .fold-btn-inline {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 12px;
		padding: 2px;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s ease;
		width: 20px;
		height: 20px;
		border-radius: 3px;
	}

	.fold-btn:hover, .fold-btn-inline:hover {
		background: var(--bg-tertiary);
		color: var(--text-primary);
	}

	.preview-box {
		border: 2px dashed var(--border-color);
		border-radius: 8px;
		padding: 80px 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-primary);
		transition: all 0.3s ease;
	}

	.preview-box:hover {
		border-color: var(--text-muted);
	}

	.preview-text {
		font-size: 24px;
		font-weight: 300;
		color: var(--text-muted);
	}

	.file-info {
		background: var(--bg-tertiary);
		border-radius: 8px;
		padding: 16px;
	}

	.info-row {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
		font-size: 13px;
		flex-wrap: wrap;
	}

	.info-row:last-child {
		margin-bottom: 0;
	}

	.info-icon {
		font-size: 16px;
	}

	.info-text {
		color: var(--text-secondary);
		flex: 1;
	}

	.info-label {
		color: var(--text-muted);
		font-size: 12px;
	}

	.info-value {
		color: var(--text-primary);
		font-size: 12px;
	}

	.file-type-tags {
		display: flex;
		gap: 6px;
		margin-left: auto;
	}

	.file-type-tag {
		background: var(--bg-secondary);
		color: var(--text-primary);
		padding: 4px 10px;
		border-radius: 12px;
		font-size: 11px;
		border: 1px solid var(--border-color);
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
		font-size: 14px;
		font-weight: 600;
		user-select: none;
	}

	.section-header.clickable {
		cursor: pointer;
		padding: 4px 8px;
		margin-left: -8px;
		border-radius: 4px;
		transition: all 0.2s ease;
	}

	.section-header.clickable:hover {
		background: var(--bg-tertiary);
	}

	.section-icon {
		font-size: 16px;
	}

	.section-count {
		font-size: 12px;
		color: var(--text-muted);
		margin-left: auto;
	}

	.section-nav {
		display: flex;
		gap: 4px;
	}

	.nav-btn {
		width: 24px;
		height: 24px;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.2rem;
		transition: all 0.2s ease;
	}

	.nav-btn:hover {
		background: var(--bg-secondary);
		transform: scale(1.05);
	}

	.add-btn {
		width: 24px;
		height: 24px;
		background: var(--btn-success);
		border: none;
		border-radius: 4px;
		color: white;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		font-weight: bold;
		transition: all 0.2s ease;
	}

	.add-btn:hover {
		background: var(--btn-success-hover);
		transform: scale(1.05);
	}

	.expand-btn {
		width: 24px;
		height: 24px;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		transition: all 0.2s ease;
	}

	.expand-btn:hover {
		background: var(--bg-secondary);
		transform: scale(1.05);
	}

	.tags-container {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.tag-item {
		display: flex;
		align-items: center;
		gap: 8px;
		background: var(--bg-tertiary);
		padding: 8px 12px;
		border-radius: 20px;
		border: 1px solid var(--border-color);
		transition: all 0.2s ease;
	}

	.tag-item:hover {
		background: var(--bg-secondary);
		box-shadow: 0 2px 8px var(--shadow);
	}

	.tag-icon {
		font-size: 14px;
	}

	.tag-name {
		font-size: 13px;
		color: var(--text-primary);
	}

	.tag-remove {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 14px;
		padding: 0;
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		transition: all 0.2s ease;
	}

	.tag-remove:hover {
		background: #ff5555;
		color: white;
	}

	.memos-container {
		display: grid;
		gap: 12px;
	}

	.memo-card {
		background: var(--bg-tertiary);
		border-radius: 8px;
		padding: 16px;
		border: 1px solid var(--border-color);
		transition: all 0.2s ease;
	}

	.memo-card:hover {
		box-shadow: 0 4px 12px var(--shadow);
	}

	.memo-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}

	.memo-title {
		margin: 0;
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.memo-actions {
		display: flex;
		gap: 4px;
	}

	.memo-action-btn {
		width: 24px;
		height: 24px;
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		transition: all 0.2s ease;
	}

	.memo-action-btn:hover {
		background: var(--bg-tertiary);
		transform: scale(1.05);
	}

	.memo-action-btn.delete:hover {
		background: #ff5555;
		color: white;
		border-color: #ff5555;
	}

	.memo-content {
		margin-bottom: 12px;
	}

	.memo-content p {
		margin: 0;
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	.memo-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.memo-tools {
		display: flex;
		gap: 8px;
	}

	.memo-tool {
		font-size: 14px;
		color: var(--text-muted);
	}

	.memo-timestamp {
		font-size: 11px;
		color: var(--text-muted);
	}

	.memo-dialog {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 12px;
		padding: 20px;
		width: 90%;
		max-width: 500px;
		box-shadow: 0 8px 32px var(--shadow);
		z-index: 10000;
	}

	.memo-dialog-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
	}

	.memo-dialog-header h4 {
		margin: 0;
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.memo-dialog-close {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 20px;
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		transition: all 0.2s ease;
	}

	.memo-dialog-close:hover {
		background: var(--bg-tertiary);
		color: var(--text-primary);
	}

	.memo-dialog-title {
		width: 100%;
		padding: 12px;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		color: var(--text-primary);
		font-size: 14px;
		margin-bottom: 12px;
		outline: none;
		transition: all 0.2s ease;
	}

	.memo-dialog-title:focus {
		border-color: var(--btn-primary);
	}

	.memo-dialog-content {
		width: 100%;
		min-height: 150px;
		padding: 12px;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		color: var(--text-primary);
		font-size: 14px;
		margin-bottom: 16px;
		resize: vertical;
		outline: none;
		font-family: inherit;
		transition: all 0.2s ease;
	}

	.memo-dialog-content:focus {
		border-color: var(--btn-primary);
	}

	.memo-dialog-footer {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.memo-dialog-btn {
		padding: 8px 20px;
		border: none;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.memo-dialog-btn.cancel {
		background: var(--bg-tertiary);
		color: var(--text-primary);
		border: 1px solid var(--border-color);
	}

	.memo-dialog-btn.cancel:hover {
		background: var(--bg-secondary);
	}

	.memo-dialog-btn.save {
		background: var(--btn-primary);
		color: white;
	}

	.memo-dialog-btn.save:hover {
		background: var(--btn-primary-hover);
		transform: scale(1.02);
	}

	.linked-container {
		display: grid;
		gap: 8px;
	}

	.linked-item {
		display: flex;
		gap: 12px;
		background: var(--bg-tertiary);
		padding: 12px;
		border-radius: 8px;
		border: 1px solid var(--border-color);
		transition: all 0.2s ease;
	}

	.linked-item:hover {
		box-shadow: 0 2px 8px var(--shadow);
	}

	.linked-icon {
		font-size: 20px;
		flex-shrink: 0;
	}

	.linked-info {
		flex: 1;
		min-width: 0;
	}

	.linked-name-row {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 4px;
		flex-wrap: wrap;
	}

	.linked-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
	}

	.linked-date {
		font-size: 11px;
		color: var(--text-muted);
	}

	.linked-tag {
		background: var(--bg-secondary);
		color: var(--text-primary);
		padding: 2px 8px;
		border-radius: 10px;
		font-size: 10px;
		border: 1px solid var(--border-color);
	}

	.linked-remove {
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 14px;
		padding: 0;
		width: 18px;
		height: 18px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		margin-left: auto;
		transition: all 0.2s ease;
	}

	.linked-remove:hover {
		background: #ff5555;
		color: white;
	}

	.linked-path-row {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 11px;
	}

	.linked-path {
		color: var(--text-muted);
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.linked-size {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	/* Navigation Bars */
	.nav-bar {
		position: absolute;
		left: 0;
		right: 0;
		background: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		padding: 8px 12px;
		display: flex;
		align-items: center;
		gap: 8px;
		z-index: 100;
		box-shadow: 0 2px 8px var(--shadow);
		flex-wrap: wrap;
	}

	.nav-bar-top {
		top: 0;
		border-radius: 0 0 8px 8px;
		border-top: none;
	}

	.nav-bar-bottom {
		bottom: 0;
		border-radius: 8px 8px 0 0;
		border-bottom: none;
	}

	.nav-bar-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-muted);
		text-transform: uppercase;
		margin-right: 4px;
	}

	.nav-bar-btn {
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		color: var(--text-primary);
		padding: 4px 12px;
		border-radius: 12px;
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.nav-bar-btn:hover {
		background: var(--btn-primary);
		color: white;
		border-color: var(--btn-primary);
		transform: scale(1.05);
	}
</style>

