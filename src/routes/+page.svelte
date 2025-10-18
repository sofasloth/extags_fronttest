<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	// 상태 관리
	let externalAI = $state({
		read: true,
		write: false,
		execute: true
	});

	let internalAI = $state({
		read: true,
		write: true,
		execute: false
	});

	// 파일 타입별 데이터
	interface FileTypeData {
		color: string;
		percentage: number;
		size: number; // MB 단위
		name: string;
	}

	let fileTypes: FileTypeData[] = $state([
		{ color: '#4ade80', percentage: 45, size: 40.5, name: 'JavaScript' },
		{ color: '#ef4444', percentage: 25, size: 22.5, name: 'CSS' },
		{ color: '#3b82f6', percentage: 20, size: 18, name: 'Images' },
		{ color: '#6b7280', percentage: 10, size: 9, name: 'Others' }
	]);

	let totalFiles = $state(11000);
	let totalSize = $state(90); // MB

	// 호버 상태 관리
	let hoveredSegment = $state<number | null>(null);
	let hoverTimeout: number | null = null;
	let showTooltip = $state(false);
	let tooltipData = $state<FileTypeData | null>(null);

	function handleMouseEnter(index: number, data: FileTypeData) {
		hoveredSegment = index;
		tooltipData = data;
		
		// 3초 후 툴팁 표시
		if (hoverTimeout) clearTimeout(hoverTimeout);
		hoverTimeout = setTimeout(() => {
			showTooltip = true;
		}, 300) as unknown as number;
	}

	function handleMouseLeave() {
		hoveredSegment = null;
		showTooltip = false;
		tooltipData = null;
		if (hoverTimeout) {
			clearTimeout(hoverTimeout);
			hoverTimeout = null;
		}
	}

	// 권한 토글 함수
	function togglePermission(ai: 'external' | 'internal', permission: 'read' | 'write' | 'execute') {
		if (ai === 'external') {
			externalAI[permission] = !externalAI[permission];
		} else {
			internalAI[permission] = !internalAI[permission];
		}
		
		// 권한 변경 시 알람창 표시
		const aiName = ai === 'external' ? '외부 AI' : '내부 AI';
		const permissionName = permission === 'read' ? '읽기' : permission === 'write' ? '쓰기' : '실행';
		const status = (ai === 'external' ? externalAI[permission] : internalAI[permission]) ? '활성화' : '비활성화';
		
		showAlertModal(`${aiName}의 ${permissionName} 권한이 ${status}되었습니다.`);
	}

	// 별도 알람창 표시 함수
	async function showAlertModal(message: string) {
		try {
			await invoke('show_alert_window', { message });
		} catch (error) {
			console.error('알람창 표시 실패:', error);
		}
	}

	// 테스트용 알람창 표시 함수
	function triggerAlert() {
		showAlertModal('권한 설정이 변경되었습니다!');
	}
</script>

<div class="container">
	<div class="folder-card">
		<!-- Header -->
		<div class="header">
			<div class="folder-icon">
				<svg viewBox="0 0 24 24" fill="currentColor">
					<path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
				</svg>
			</div>
			<div class="header-text">
				<h2>Documents</h2>
			</div>
		</div>

		<!-- Path row -->
		<div class="path-row">
			<span class="token-label">토큰</span>
			<span class="path">C:/Users/username/Documents</span>
		</div>

		<!-- Permission rows -->
		<div class="permission-row">
			<span class="ai-label">외부 AI</span>
			<div class="permissions">
				<button 
					class="permission-btn"
					class:active={externalAI.read}
					onclick={() => togglePermission('external', 'read')}
				>
					읽기
				</button>
				<button 
					class="permission-btn"
					class:active={externalAI.write}
					onclick={() => togglePermission('external', 'write')}
				>
					쓰기
				</button>
				<button 
					class="permission-btn"
					class:active={externalAI.execute}
					onclick={() => togglePermission('external', 'execute')}
				>
					실행
				</button>
			</div>
		</div>

		<div class="permission-row">
			<span class="ai-label">내부 AI</span>
			<div class="permissions">
				<button 
					class="permission-btn"
					class:active={internalAI.read}
					onclick={() => togglePermission('internal', 'read')}
				>
					읽기
				</button>
				<button 
					class="permission-btn"
					class:active={internalAI.write}
					onclick={() => togglePermission('internal', 'write')}
				>
					쓰기
				</button>
				<button 
					class="permission-btn"
					class:active={internalAI.execute}
					onclick={() => togglePermission('internal', 'execute')}
				>
					실행
				</button>
			</div>
		</div>

		<!-- Bottom gauge bar -->
		<div class="bottom-section">
			<div class="gauge-container">
				<span class="share-label">share</span>
				<div class="gauge-bar">
					{#each fileTypes as fileType, index}
						<div 
							class="gauge-segment"
							style="width: {fileType.percentage}%; background-color: {fileType.color};"
							onmouseenter={() => handleMouseEnter(index, fileType)}
							onmouseleave={handleMouseLeave}
							role="button"
							tabindex="0"
						>
							{#if showTooltip && hoveredSegment === index && tooltipData}
								<div class="tooltip">
									<div class="tooltip-content">
										<strong>{tooltipData.name}</strong>
										<div>{tooltipData.percentage}%</div>
										<div>{tooltipData.size.toFixed(1)} MB</div>
									</div>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
			<div class="stats">
				<div class="stat-item">
					<span class="stat-value">{totalFiles.toLocaleString()}</span>
				</div>
				<div class="stat-item">
					<span class="stat-value">{totalSize} MB</span>
				</div>
			</div>
		</div>

		<!-- 테스트용 알람 버튼 -->
		<div class="alert-button-container">
			<button class="alert-trigger-btn" onclick={triggerAlert}>
				알람 테스트
			</button>
		</div>
	</div>
</div>

<style>
	.container {
		display: flex;
		justify-content: center;
		align-items: center;
		min-height: calc(100vh - 60px);
		padding: 2rem;
		background: var(--bg-primary);
		margin-top: 60px;
	}

	.folder-card {
		width: 100%;
		max-width: 650px;
		background: linear-gradient(135deg, #2d2d2d 0%, #1a1a1a 100%);
		border-radius: 16px;
		padding: 1.5rem;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	/* Header */
	.header {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding-bottom: 0.5rem;
	}

	.folder-icon {
		width: 48px;
		height: 48px;
		background: linear-gradient(135deg, #ff8a00 0%, #e52e71 100%);
		border-radius: 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: white;
	}

	.folder-icon svg {
		width: 28px;
		height: 28px;
	}

	.header-text h2 {
		margin: 0;
		font-size: 1.8rem;
		font-weight: 600;
		color: #ffffff;
	}

	/* Path row */
	.path-row {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem 1rem;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 8px;
	}

	.token-label {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
		padding: 0.4rem 0.8rem;
		border-radius: 6px;
		font-size: 0.85rem;
		font-weight: 600;
	}

	.path {
		color: #cccccc;
		font-size: 0.9rem;
		font-family: 'Courier New', monospace;
	}

	/* Permission rows */
	.permission-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.75rem 1rem;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 8px;
	}

	.ai-label {
		color: #ffffff;
		font-weight: 600;
		font-size: 1rem;
		min-width: 80px;
	}

	.permissions {
		display: flex;
		gap: 0.75rem;
	}

	.permission-btn {
		padding: 0.5rem 1.2rem;
		border: none;
		border-radius: 6px;
		font-size: 0.9rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.3s ease;
		background: #ef4444;
		color: white;
		box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
	}

	.permission-btn.active {
		background: #22c55e;
		box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
	}

	.permission-btn:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	/* Bottom section */
	.bottom-section {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding-top: 0.5rem;
	}

	.gauge-container {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.share-label {
		color: #999999;
		font-size: 0.85rem;
		font-weight: 500;
	}

	.gauge-bar {
		display: flex;
		height: 24px;
		background: rgba(0, 0, 0, 0.3);
		border-radius: 12px;
		overflow: hidden;
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.2);
		position: relative;
	}

	.gauge-segment {
		position: relative;
		cursor: pointer;
		transition: all 0.3s ease;
	}

	.gauge-segment:hover {
		filter: brightness(1.2);
	}

	.gauge-segment:first-child {
		border-top-left-radius: 12px;
		border-bottom-left-radius: 12px;
	}

	.gauge-segment:last-child {
		border-top-right-radius: 12px;
		border-bottom-right-radius: 12px;
	}

	/* Tooltip */
	.tooltip {
		position: absolute;
		bottom: 100%;
		left: 50%;
		transform: translateX(-50%);
		margin-bottom: 10px;
		z-index: 1000;
		animation: fadeIn 0.3s ease;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateX(-50%) translateY(5px);
		}
		to {
			opacity: 1;
			transform: translateX(-50%) translateY(0);
		}
	}

	.tooltip-content {
		background: rgba(0, 0, 0, 0.9);
		color: white;
		padding: 0.75rem 1rem;
		border-radius: 8px;
		font-size: 0.85rem;
		white-space: nowrap;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	.tooltip-content strong {
		display: block;
		margin-bottom: 0.25rem;
		color: #4ade80;
	}

	.tooltip-content div {
		line-height: 1.4;
	}

	/* Stats */
	.stats {
		display: flex;
		gap: 0.75rem;
	}

	.stat-item {
		background: rgba(255, 255, 255, 0.05);
		padding: 0.6rem 1rem;
		border-radius: 8px;
		text-align: center;
		min-width: 80px;
	}

	.stat-value {
		color: #ffffff;
		font-weight: 600;
		font-size: 0.95rem;
	}

	/* 반응형 */
	@media (max-width: 640px) {
		.container {
			padding: 1rem;
		}

		.folder-card {
			padding: 1rem;
		}

		.header-text h2 {
			font-size: 1.4rem;
		}

		.permissions {
			flex-direction: column;
			gap: 0.5rem;
		}

		.permission-btn {
			padding: 0.4rem 0.8rem;
			font-size: 0.85rem;
		}

		.bottom-section {
			flex-direction: column;
			align-items: stretch;
		}

		.stats {
			justify-content: center;
		}
	}

	/* 테스트용 알람 버튼 */
	.alert-button-container {
		display: flex;
		justify-content: center;
		margin-top: 1rem;
	}

	.alert-trigger-btn {
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
		border: none;
		padding: 0.75rem 1.5rem;
		border-radius: 8px;
		font-size: 0.9rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.3s ease;
		box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
	}

	.alert-trigger-btn:hover {
		transform: translateY(-2px);
		box-shadow: 0 6px 16px rgba(102, 126, 234, 0.4);
	}

</style>
