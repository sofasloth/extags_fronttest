<script lang="ts">
	import { onMount } from 'svelte';
	import {slide, scale} from 'svelte/transition';
	// import {InOut} from 'svelte/easing';
	// 상태 관리
	let content = $state('');
	let memoName = $state('새 메모');
	let currentDateTime = $state('');
	let isExpanded = $state(false);
	let showTagPanel = $state(false);
	let showAlarmPanel = $state(false);

	// 태그 관련
	let tags = $state<string[]>([]);
	let tagInput = $state('');
	const MAX_TAGS = 5;

	// 알람 관련
	let alarmName = $state('');
	let alarmDate = $state('');
	let alarmTime = $state('09:10');
	let alarmPeriod = $state<'morning' | 'afternoon' | 'evening' | 'night'>('morning');

	// 리사이즈 관련
	let isResizing = $state(false);
	let containerElement: HTMLDivElement;
	let startX = 0;
	let startY = 0;
	let startWidth = 0;
	let startHeight = 0;

	// 자동 높이 조절 관련
	let textareaElement: HTMLTextAreaElement;

	function updateDateTime() {
		const now = new Date();
		const year = now.getFullYear();
		const month = String(now.getMonth() + 1).padStart(2, '0');
		const day = String(now.getDate()).padStart(2, '0');
		const hours = now.getHours();
		const minutes = String(now.getMinutes()).padStart(2, '0');
		const period = hours >= 12 ? 'PM' : 'AM';
		const displayHours = hours % 12 || 12;

		currentDateTime = `${year}-${month}-${day} ${displayHours}:${minutes} ${period}`;
	}

	onMount(() => {
		updateDateTime();
		const interval = setInterval(updateDateTime, 60000);

		// 기본 날짜 설정
		alarmDate = getTodayDate();

		// ESC 키 이벤트 리스너
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				showTagPanel = false;
				showAlarmPanel = false;
			}
		};
		window.addEventListener('keydown', handleKeyDown);

		// 초기 높이 조절
		setTimeout(() => {
			console.log('onMount: calling adjustTextareaHeight');
			adjustTextareaHeight();
		}, 100);

		return () => {
			clearInterval(interval);
			window.removeEventListener('keydown', handleKeyDown);
		};
	});

	function toggleTheme() {
		if (typeof window !== 'undefined' && (window as any).__toggleTheme) {
			(window as any).__toggleTheme();
		}
	}

	function toggleTagPanel() {
		showTagPanel = !showTagPanel;
		if (showTagPanel) {
			showAlarmPanel = false;
		}
	}

	function toggleAlarmPanel() {
		showAlarmPanel = !showAlarmPanel;
		if (showAlarmPanel) {
			showTagPanel = false;
		}
	}

	function toggleExpanded() {
		isExpanded = !isExpanded;
		showTagPanel = false;
		showAlarmPanel = false;
	}

	function addTag() {
		if (tagInput.trim() && tags.length < MAX_TAGS && !tags.includes(tagInput.trim())) {
			tags = [...tags, tagInput.trim()];
			tagInput = '';
		}
	}

	function removeTag(index: number) {
		tags = tags.filter((_, i) => i !== index);
	}

	function handleTagKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addTag();
		}
	}

	function getFormattedAlarmDate() {
		if (!alarmDate) return '';
		const date = new Date(alarmDate);
		const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
		const day = String(date.getDate()).padStart(2, '0');
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const year = String(date.getFullYear()).slice(2);
		return `${day}/${month}/${year} ${days[date.getDay()]}`;
	}

	function calculateDaysUntil() {
		if (!alarmDate) return 'D+00';
		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const target = new Date(alarmDate);
		target.setHours(0, 0, 0, 0);
		const diff = Math.floor((target.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
		return diff >= 0 ? `D-${String(diff).padStart(2, '0')}` : `D+${String(diff).padStart(2, '0')}`;
	}

	// 오늘 날짜를 YYYY-MM-DD 형식으로 반환
	function getTodayDate() {
		const today = new Date();
		const year = today.getFullYear();
		const month = String(today.getMonth() + 1).padStart(2, '0');
		const day = String(today.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	// 날짜를 YYYY/MM/DD 형식으로 변환
	function formatDateWithSlash(dateString: string) {
		if (!dateString) return '';
		return dateString.replace(/-/g, '/');
	}

	// 날짜에서 요일 반환
	function getDayOfWeek(dateString: string) {
		if (!dateString) return '';
		const date = new Date(dateString);
		const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
		return days[date.getDay()];
	}


	// // 리사이즈 핸들러
	// function handleResizeStart(e: MouseEvent) {
	// 	e.preventDefault();
	// 	isResizing = true;
	// 	startX = e.clientX;
	// 	startY = e.clientY;
	// 	startWidth = containerElement.offsetWidth;
	// 	startHeight = containerElement.offsetHeight;
		
	// 	document.addEventListener('mousemove', handleResizeMove);
	// 	document.addEventListener('mouseup', handleResizeEnd);
	// }

	// function handleResizeMove(e: MouseEvent) {
	// 	if (!isResizing) return;
		
	// 	const deltaX = e.clientX - startX;
	// 	const deltaY = e.clientY - startY;
		
	// 	const newWidth = Math.max(240, Math.min(720, startWidth + deltaX));
	// 	const newHeight = Math.max(120, Math.min(window.innerHeight * 0.8, startHeight + deltaY));
		
	// 	containerElement.style.width = `${newWidth}px`;
	// 	containerElement.style.height = `${newHeight}px`;
	// }

	// function handleResizeEnd() {
	// 	isResizing = false;
	// 	document.removeEventListener('mousemove', handleResizeMove);
	// 	document.removeEventListener('mouseup', handleResizeEnd);
	// }

	// 자동 높이 조절 함수
	function adjustTextareaHeight() {
		if (!textareaElement) {
			console.log('adjustTextareaHeight: textareaElement not found');
			return;
		}
		
		// 스크롤 높이를 0으로 설정하여 실제 내용 높이를 측정
		textareaElement.style.height = 'auto';
		const scrollHeight = textareaElement.scrollHeight;
		
		console.log('adjustTextareaHeight: scrollHeight =', scrollHeight);
		
		// 최소 높이 설정
		const minHeight = 24; // 1.5em 정도
		const newHeight = Math.max(minHeight, scrollHeight);
		
		textareaElement.style.height = `${newHeight}px`;
	}

	// 텍스트 변경 시 높이 조절
	function handleTextareaInput() {
		console.log('handleTextareaInput called');
		adjustTextareaHeight();
	}

	// 메모 콘텐츠 영역 클릭 시 textarea 포커스
	function handleContentClick() {
		if (textareaElement) {
			textareaElement.focus();
		}
	}

	// 키보드 이벤트 핸들러
	function handleContentKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			// e.preventDefault();
			if (textareaElement) {
				textareaElement.focus();
			}
		}
	}
</script>

<div class="memo-container" class:expanded={isExpanded} bind:this={containerElement}>
	<header class="memo-header">
		<div class="header-left">
			{#if isExpanded && alarmName}
				<span class="alarm-icon">⏰</span>
			{/if}
			<!-- <h1>새 메모</h1> -->
			 <input type="text" bind:value={memoName} placeholder="새 메모" class="memo-name-input" />
		</div>
		<div class="window-controls">
			<button class="control-btn theme-btn" onclick={toggleTheme} title="Toggle theme">
				🎨
			</button>
			<button class="control-btn minimize">−</button>
			<button class="control-btn maximize" onclick={toggleExpanded}>□</button>
			<button class="control-btn close">×</button>
		</div>
	</header>

	{#if isExpanded}
		<!-- 확장 모드 -->
		<div class="expanded-header" in:slide={{duration: 200}} out:slide={{duration: 200}}>
			<div class="expanded-tags">
				<button class="meta-btn">#</button>
				{#each tags as tag}
					<span class="expanded-tag">{tag}</span>
				{/each}
			</div>
			<div class="expanded-datetime">{currentDateTime}</div>
		</div>

		<div class="expanded-meta" in:slide={{duration: 200}} out:slide={{duration: 200}}>
			<button class="meta-btn">@</button>
			{#if alarmName}
				<button class="meta-btn alarm">⏰</button>
			{/if}
		</div>

		{#if alarmName}
			<div class="expanded-alarm-info">
				<div class="alarm-details">
					<span class="alarm-icon-small">⏰</span>
					<span class="alarm-name">{alarmName}</span>
				</div>
				<div class="alarm-schedule">
					<span class="alarm-queue">Next Queue : {calculateDaysUntil()}</span>
					<div class="alarm-datetime">
						<span class="alarm-date">{getFormattedAlarmDate()}</span>
						<span class="alarm-time">{alarmTime} {alarmPeriod}</span>
					</div>
				</div>
			</div>
		{/if}
	{/if}

	<div class="memo-content-wrapper">
		<div 
			class="memo-content" 
			onclick={handleContentClick}
			onkeydown={handleContentKeydown}
			role="button"
			tabindex="0"
			title="Click to focus text area"
		>
			<textarea
				bind:value={content}
				bind:this={textareaElement}
				placeholder="아무 내용을 채워주세요."
				class="memo-textarea"
				oninput={handleTextareaInput}
			></textarea>
		</div>

		{#if showTagPanel || showAlarmPanel}
			<div 
				class="panel-overlay" 
				role="button"
				tabindex="0"
				onclick={() => { showTagPanel = false; showAlarmPanel = false; }}
				onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { showTagPanel = false; showAlarmPanel = false; } }}
			></div>
		{/if}

		{#if showTagPanel}
			<div class="input-panel tag-panel" in:slide={{duration: 200, delay: 200}} out:slide={{duration: 200}}>
				<div class="panel-header">
					<span class="panel-icon">#</span>
					<input
						type="text"
						bind:value={tagInput}
						onkeydown={handleTagKeydown}
						placeholder="Tag name"
						class="panel-input"
						maxlength={20}
					/>
				</div>
				<div class="tag-list">
					{#each tags as tag, index}
						<button class="tag-item" onclick={() => removeTag(index)}>
							<span class="tag-icon">⚙️</span>
							<span class="tag-text">{tag}</span>
							<span class="tag-remove">×</span>
						</button>
					{/each}
				</div>
				<div class="panel-footer">
					<span>add tag / link</span>
					<span>• 최대 {MAX_TAGS}개 검색</span>
				</div>
			</div>
		{/if}

		{#if showAlarmPanel}
			<div class="input-panel alarm-panel" in:slide={{duration: 200, delay: 200}} out:slide={{duration: 200}}>
				<div class="alarm-form">
					<div class="alarm-form-row">
						<div class="alarm-icon-header">⏰</div>
						<input
							type="text"
							bind:value={alarmName}
							placeholder="Alarm_name"
							class="alarm-name-input"
						/>
					</div>
					<div class="alarm-form-row">
						<div class="alarm-queue-display">Next Queue : {calculateDaysUntil()}</div>
						<div class="flex-spacer"></div>
						<select bind:value={alarmPeriod} class="alarm-period-select">
							<option value="morning">morning</option>
							<option value="afternoon">afternoon</option>
							<option value="evening">evening</option>
							<option value="night">night</option>
						</select>
					</div>
					<div class="alarm-form-row">
							<input 
								type="date" 
								bind:value={alarmDate}
								class="alarm-date-input" 
								placeholder="YYYY/MM/DD"
							/>
							<span class="alarm-date-separator">{getDayOfWeek(alarmDate)}</span>
							<div class="flex-spacer"></div>
							<div class="alarm-time-display">
								<input type="time" bind:value={alarmTime} class="alarm-time-input" />
							</div>
					</div>
				</div>
			</div>
		{/if}
	</div>

	{#if !isExpanded && !showTagPanel && !showAlarmPanel}
		<footer class="memo-footer" in:slide={{duration: 200, delay: 200}} out:slide={{duration: 200}}>
			<div class="toolbar">
				<button
					class="tool-btn"
					class:active={showTagPanel}
					onclick={toggleTagPanel}
					title="해시태그"
				>
					#
				</button>
				<button class="tool-btn" onclick={() => (content += '@')} title="멘션">@</button>
				<button
					class="tool-btn alarm-btn"
					class:active={showAlarmPanel}
					onclick={toggleAlarmPanel}
					title="알람"
				>
					⏰
				</button>
			</div>
			<div class="datetime">{currentDateTime}</div>
		</footer>
	{/if}
</div>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		display: flex;
		justify-content: center;
		align-items: center;
		min-height: 100vh;
	}

	.memo-container {
		display: flex;
		flex-direction: column;
		min-width: 240px;
		max-width: 720px;
		width: 90vw;
		min-height: 240px;
		max-height: 80vh;
		background-color: var(--bg-secondary);
		color: var(--text-primary);
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue',
			Arial, sans-serif;
		border-radius: 12px;
		box-shadow: 0 4px 24px var(--shadow);
		overflow: hidden;
		border: 2px solid var(--border-color);
		transition: background-color 0.3s ease;
		position: relative;
		/* resize: both; */
	}

	.memo-container.expanded {
		max-height: 90vh;
		height: 90vh;
	}

	.memo-container:hover,
	.memo-container:focus-within {
		border: 2px solid var(--accent-border);
		box-shadow: 0 4px 12px var(--shadow);
		transition: border-color 0.3s ease;
	}

	.memo-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.4rem 0.4rem;
		background-color: var(--bg-primary);
		/* border-bottom: 1px solid var(--border-color); */
		flex-shrink: 0;
		transition: all 0.3s ease;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 8px;
		flex : 1;
	}

	.alarm-icon {
		font-size: 1rem;
	}

	.memo-header input.memo-name-input {
		width: 100%;
		height: 100%;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-size: 1.5rem;
		font-weight: 700;
		background-color: transparent;
		word-wrap: normal;
	}

	.window-controls {
		display: flex;
		gap: 8px;
	}

	.control-btn {
		width: 24px;
		height: 24px;
		/* border: 1px solid var(--border-color); */
		border: none;
		border-radius: 4px;
		font-size: 18px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
		color: var(--text-primary);
		background-color: var(--bg-primary);
	}

	.control-btn:hover {
		background-color: var(--bg-secondary);
		transform: scale(1.05);
	}

	.control-btn.theme-btn {
		background-color: var(--btn-primary);
		color: white;
		border-color: var(--btn-primary);
	}

	.control-btn.theme-btn:hover {
		background-color: var(--btn-primary-hover);
		border-color: var(--btn-primary-hover);
	}

	.control-btn.minimize {
		color: var(--text-primary);
	}

	.control-btn.maximize {
		color: var(--text-primary);
	}

	.control-btn.close {
		color: var(--text-primary);
	}

	/* 확장 모드 헤더 */
	.expanded-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.4rem 0.4rem;
		background-color: var(--bg-primary);
		/* border-bottom: 1px solid var(--border-color); */
	}

	.expanded-tags {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}


	.expanded-tag {
		padding: 4px 12px;
		background-color: #e0e0e0;
		color: #333;
		border-radius: 16px;
		font-size: 0.85rem;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.expanded-datetime {
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.expanded-meta {
		display: flex;
		gap: 8px;
		padding: 0.4rem 0.4rem;
		background-color: var(--bg-primary);
	}

	.meta-btn {
		width: 24px;
		height: 24px;
		border: 1px solid var(--border-color);
		border-radius: 6px;
		background-color: var(--bg-tertiary);
		color: var(--text-primary);
		font-size: 1rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
	}

	.meta-btn:hover {
		background-color: var(--bg-primary);
		transform: scale(1.05);
	}

	.expanded-alarm-info {
		padding: 12px 20px;
		background-color: var(--bg-secondary);
		border-bottom: 1px solid var(--border-color);
	}

	.alarm-details {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
	}

	.alarm-icon-small {
		font-size: 0.9rem;
	}

	.alarm-name {
		color: var(--text-primary);
		font-size: 0.95rem;
	}

	.alarm-schedule {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
	}

	.alarm-queue {
		color: var(--text-muted);
		font-size: 0.85rem;
	}

	.alarm-datetime {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 4px;
	}

	.alarm-date {
		color: var(--text-primary);
		font-size: 1.2rem;
		font-weight: 500;
	}

	.alarm-time {
		color: var(--text-primary);
		font-size: 1.8rem;
		font-weight: 600;
	}

	/* 메모 콘텐츠 */
	.memo-content-wrapper {
		flex: 1;
		position: relative;
		overflow: hidden;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}

	.memo-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 0.2rem 0.4rem;
		overflow-y: auto;
		overflow-x: hidden;
		background-color: var(--bg-primary);
		transition: background-color 0.3s ease;
		min-height: 0;
		max-height: 100%;
		cursor: text;
	}

	.memo-content::-webkit-scrollbar {
		width: 8px;
	}

	.memo-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.memo-content::-webkit-scrollbar-thumb {
		background-color: var(--border-color);
		border-radius: 4px;
	}

	.memo-content::-webkit-scrollbar-thumb:hover {
		background-color: var(--text-muted);
	}

	.memo-textarea {
		width: 100%;
		flex: 1;
		min-height: 0;
		background-color: transparent;
		border: none;
		outline: none;
		color: var(--text-secondary);
		font-size: 16px;
		font-family: inherit;
		resize: none;
		line-height: 1.6;
		word-wrap: break-word;
		overflow-wrap: break-word;
		white-space: pre-wrap;
		transition: color 0.3s ease;
		overflow-y: auto;
		height: auto;
		min-height: 1.6em;
	}

	.memo-textarea::placeholder {
		color: var(--text-muted);
	}

	/* 패널 오버레이 */
	.panel-overlay {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 40%;
		background-color: var(--bg-primary);
		pointer-events: none;
		z-index: 1;
	}

	/* 입력 패널 */
	.input-panel {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		background: linear-gradient(to bottom, var(--bg-primary) 0%, var(--bg-tertiary) 100%);
		background-color: var(--bg-primary);
		/* border-top: 1px solid var(--border-color); */
		padding: 0.2rem 0.4rem;
		z-index: 2;
		transition: background 0.3s ease;
	}

	/* 태그 패널 */
	.tag-panel {
		max-height: 280px;
	}

	.panel-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
	}

	.panel-icon {
		font-size: 1.2rem;
		color: var(--text-primary);
	}

	.panel-input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-size: 1rem;
		padding: 8px 0;
	}

	.panel-input::placeholder {
		color: var(--text-muted);
	}

	.tag-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-bottom: 12px;
	}

	.tag-item {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		padding: 0.2rem 0.4rem;
		background-color: #e0e0e0;
		color: #000;
		border: none;
		border-radius: 20px;
		font-size: 0.9rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.tag-item:hover {
		background-color: #d0d0d0;
	}

	.tag-icon {
		width: 1.1rem;
		font-size: 0.9rem;
	}

	.tag-text {
		font-weight: 500;
	}

	.tag-remove {
		width: 1.1rem;
		font-size: 1.2rem;
		font-weight: 600;
		/* margin-left: 2px; */
	}
	
	.tag-remove:hover {
		color: var(--status-error-border);
		transform: scale(1.1);
		transition: all 0.2s ease;
	}

	.panel-footer {
		display: flex;
		gap: 8px;
		color: var(--text-muted);
		font-size: 0.85rem;
	}

	/* 알람 패널 */
	.alarm-panel {
		max-height: 320px;
	}

	.alarm-form {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
	}

	.alarm-form-row {
		display: flex;
		flex-direction: row;
		gap: 8px;
		align-items: end;
	}

	.alarm-icon-header {
		font-size: 1.2rem;
	}

	.alarm-name-input {
		background: transparent;
		border: none;
		border-bottom: 1px solid var(--border-color);
		outline: none;
		color: var(--text-primary);
		font-size: 1rem;
		font-family: inherit;
		padding: 0.2rem 0;
	}

	.alarm-name-input::placeholder {
		color: var(--text-muted);
	}

	.alarm-queue-display {
		color: var(--text-muted);
		font-size: 0.9rem;
	}

	.flex-spacer {
		flex: 1;
		height:auto;
	}


	.alarm-date-input,
	.alarm-time-input {
		background-color: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-family: inherit;
		font-size: 1.5rem;
		font-weight: 600;
		padding: 0.2rem 0;
	}

	.alarm-date-input {
		flex-shrink: 0;
		flex-grow: 0;
	}

	.alarm-date-separator {
		font-size: 1.2rem;
		font-weight: 500;
		color: var(--text-muted);
		margin-bottom: 0.2rem;
	}

	.alarm-period-select {
		background-color: var(--bg-tertiary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		color: var(--text-primary);
		padding: 0.2rem 0.4rem;
		font-size: 0.9rem;
		outline: none;
	}

	.alarm-time-display {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.alarm-time-input {
		flex: 1;
	}

	.alarm-period-select {
		padding: 8px;
	}

	/* Footer */
	.memo-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.2rem 0.4rem;
		background-color: var(--bg-primary);
		/* border-top: 1px solid var(--border-color); */
		flex-shrink: 0;
		transition: all 0.3s ease;
	}

	.toolbar {
		margin-left: 0.4rem;
		display: flex;
		gap: 12px;
	}

	.tool-btn {
		width: 24px;
		height: 24px;
		border: 1px solid var(--border-color);
		border-radius: 6px;
		background-color: var(--bg-secondary);
		color: var(--text-primary);
		font-size: 18px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
	}

	.tool-btn:hover {
		background-color: var(--bg-tertiary);
		transform: scale(1.05);
	}

	.tool-btn.active {
		background-color: var(--btn-primary);
		color: white;
		border-color: var(--btn-primary);
	}

	.alarm-btn {
		font-size: 16px;
	}

	.datetime {
		color: var(--text-muted);
		font-size: 14px;
	}
</style>
