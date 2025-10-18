<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';

	// 알람창 상태
	let alertMessage = $state('');
	let isVisible = $state(false);

	// Tauri 이벤트 리스너
	async function setupEventListeners() {
		// 메시지 수신 이벤트
		await listen<string>('show-alert', (event) => {
			alertMessage = event.payload;
			showAlert();
		});

		// 창 닫기 이벤트
		await listen('close-alert', () => {
			closeAlert();
		});
	}

	// 알람창 표시
	function showAlert() {
		isVisible = true;
		// 우측 상단으로 위치 조정
		positionWindow();
	}

	// 알람창 닫기
	async function closeAlert() {
		isVisible = false;
		// 메인 창에 닫기 완료 알림
		await invoke('alert_closed');
	}

	// 창 위치 조정 (우측 상단)
	async function positionWindow() {
		try {
			await invoke('position_alert_window');
		} catch (error) {
			console.error('창 위치 조정 실패:', error);
		}
	}

	// 컴포넌트 마운트 시 이벤트 리스너 설정
	$effect(() => {
		setupEventListeners();
	});
</script>

{#if isVisible}
	<div class="alert-container">
		<div class="mac-alert">
			<div class="alert-header">
				<div class="alert-icon">
					<svg viewBox="0 0 24 24" fill="currentColor">
						<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/>
					</svg>
				</div>
				<button 
					class="alert-close" 
					onclick={closeAlert}
					aria-label="알림 닫기"
				>
					<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
						<path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
					</svg>
				</button>
			</div>
			<div class="alert-content">
				<div class="alert-title">알림</div>
				<div class="alert-message">{alertMessage}</div>
			</div>
			<div class="alert-actions">
				<button class="alert-btn alert-btn-primary" onclick={closeAlert}>
					확인
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		background: transparent;
		overflow: hidden;
	}

	.alert-container {
		width: 100vw;
		height: 100vh;
		display: flex;
		justify-content: flex-end;
		align-items: flex-start;
		padding: 1rem;
		background: transparent;
	}

	.mac-alert {
		background: rgba(255, 255, 255, 0.95);
		backdrop-filter: blur(20px);
		border-radius: 12px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
		border: 1px solid rgba(255, 255, 255, 0.2);
		min-width: 320px;
		max-width: 400px;
		animation: slideInFromRight 0.4s cubic-bezier(0.16, 1, 0.3, 1);
		position: relative;
		overflow: hidden;
	}

	@keyframes slideInFromRight {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.alert-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.25rem 0.5rem;
	}

	.alert-icon {
		width: 24px;
		height: 24px;
		color: #22c55e;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.alert-icon svg {
		width: 20px;
		height: 20px;
	}

	.alert-close {
		background: none;
		border: none;
		color: #6b7280;
		cursor: pointer;
		padding: 0.25rem;
		border-radius: 4px;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.alert-close:hover {
		background: rgba(0, 0, 0, 0.05);
		color: #374151;
	}

	.alert-close svg {
		width: 16px;
		height: 16px;
	}

	.alert-content {
		padding: 0 1.25rem 1rem;
	}

	.alert-title {
		font-size: 1.1rem;
		font-weight: 600;
		color: #111827;
		margin-bottom: 0.5rem;
	}

	.alert-message {
		font-size: 0.95rem;
		color: #4b5563;
		line-height: 1.5;
	}

	.alert-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		padding: 0 1.25rem 1.25rem;
	}

	.alert-btn {
		padding: 0.5rem 1.25rem;
		border: none;
		border-radius: 6px;
		font-size: 0.9rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		min-width: 80px;
	}

	.alert-btn-primary {
		background: #3b82f6;
		color: white;
		box-shadow: 0 2px 4px rgba(59, 130, 246, 0.2);
	}

	.alert-btn-primary:hover {
		background: #2563eb;
		transform: translateY(-1px);
		box-shadow: 0 4px 8px rgba(59, 130, 246, 0.3);
	}

	.alert-btn-primary:active {
		transform: translateY(0);
	}
</style>

