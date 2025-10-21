<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';

	// 알람창 상태
	let alertMessage = $state('');
	let isVisible = $state(false);
	let timeRemaining = $state(0);
	
	// 3D 효과를 위한 마우스 위치
	let mouseX = $state(0);
	let mouseY = $state(0);
	let isHovering = $state(false);

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
		// 백엔드에서 자동 닫기 설정 (5초)
		setupBackendAutoClose();
	}

	// 알람창 닫기
	async function closeAlert() {
		isVisible = false;
		// 메인 창에 닫기 완료 알림
		await invoke('alert_closed');
	}

	// 백엔드에서 자동 닫기 설정
	async function setupBackendAutoClose() {
		try {
			// 백엔드에서 5초 후 자동으로 창을 닫도록 설정
			await invoke('auto_close_alert_window', { delaySeconds: 5  });
			
			// 프론트엔드에서 카운트다운 표시
			startCountdownDisplay();
		} catch (error) {
			console.error('자동 닫기 설정 실패:', error);
		}
	}

	// 카운트다운 표시용 (백엔드에서 실제 닫기는 처리)
	function startCountdownDisplay() {
		timeRemaining = 5; // 300초
		
		// 1초마다 남은 시간 업데이트
		const countdownInterval = setInterval(() => {
			timeRemaining -= 1;
			if (timeRemaining <= 0) {
				clearInterval(countdownInterval);
			}
		}, 1000);
	}

	// 창 위치 조정 (우측 상단)
	async function positionWindow() {
		try {
			await invoke('position_alert_window');
		} catch (error) {
			console.error('창 위치 조정 실패:', error);
		}
	}

	// 마우스 이벤트 핸들러
	function handleMouseMove(event: MouseEvent) {
		if (!isHovering) return;
		
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const centerX = rect.left + rect.width / 2;
		const centerY = rect.top + rect.height / 2;
		
		mouseX = (event.clientX - centerX) / (rect.width / 2);
		mouseY = (event.clientY - centerY) / (rect.height / 2);
	}
	
	async function handleMouseEnter() {
		isHovering = true;
		// alert 박스에 마우스가 들어오면 상호작용 가능하게
		try {
			await invoke('enable_alert_interaction');
		} catch (error) {
			console.error('상호작용 활성화 실패:', error);
		}
	}
	
	async function handleMouseLeave() {
		isHovering = false;
		mouseX = 0;
		mouseY = 0;
		// alert 박스에서 마우스가 나가면 클릭 관통
		try {
			await invoke('disable_alert_interaction');
		} catch (error) {
			console.error('상호작용 비활성화 실패:', error);
		}
	}

	// 컴포넌트 마운트 시 이벤트 리스너 설정
	$effect(() => {
		setupEventListeners();
	});
	
</script>

{#if isVisible}
	<!-- <div class="alert-container"> -->
		<div 
			class="alert-3d-container"
			role="presentation"
			aria-label="3D interactive alert container"
			onmousemove={handleMouseMove}
			onmouseenter={handleMouseEnter}
			onmouseleave={handleMouseLeave}
			style:transform="rotateX({mouseY * 15}deg) rotateY({mouseX * 15}deg)"
		>
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
				{#if timeRemaining > 0}
					<div class="auto-close-countdown">
						{timeRemaining}초 후 자동으로 닫힙니다
					</div>
				{/if}
			</div>
			<div class="alert-actions">
				<button class="alert-btn alert-btn-primary" onclick={closeAlert}>
					확인
				</button>
			</div>
			</div>
		</div>
	<!-- </div> -->
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
		justify-content: center;
		align-items: flex-start;
		/* padding: 1rem; */
		background: transparent;
	}

	.alert-3d-container {
		perspective: 1000px;
		transform-style: preserve-3d;
		transition: transform 0.1s ease-out;
		cursor: pointer;
		pointer-events: auto;
		/* z-index: 1000; */
	}

	.mac-alert {
		background: linear-gradient(135deg, 
			rgba(255, 255, 255, 1.0) 0%, 
			rgba(255, 255, 255, 0.9) 100%);
		backdrop-filter: blur(25px) saturate(180%);
		-webkit-backdrop-filter: blur(25px) saturate(180%);
		border-radius: 16px;
		box-shadow: 
			0 8px 32px rgba(0, 0, 0, 0.12),
			0 2px 16px rgba(0, 0, 0, 0.08),
			inset 0 1px 0 rgba(255, 255, 255, 0.4);
		border: 1px solid rgba(255, 255, 255, 0.18);
		/* min-width: 320px; */
		/* max-width: 400px; */
		width: 100%;
		height: 100%;
		animation: slideInFromRight 0.4s cubic-bezier(0.16, 1, 0.3, 1);
		position: relative;
		overflow: hidden;
		transform-style: preserve-3d;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}
	
	.alert-3d-container:hover .mac-alert {
		box-shadow: 
			0 20px 40px rgba(0, 0, 0, 0.15),
			0 8px 24px rgba(0, 0, 0, 0.1),
			inset 0 1px 0 rgba(255, 255, 255, 0.5);
		background: linear-gradient(135deg, 
			rgba(255, 255, 255, 0.95) 0%, 
			rgba(255, 255, 255, 0.85) 100%);
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
		background: rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(10px);
		border: 1px solid rgba(255, 255, 255, 0.2);
		color: #6b7280;
		cursor: pointer;
		padding: 0.5rem;
		border-radius: 8px;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		overflow: hidden;
	}

	.alert-close::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: linear-gradient(45deg, 
			rgba(255, 255, 255, 0.1), 
			rgba(255, 255, 255, 0.05));
		opacity: 0;
		transition: opacity 0.3s ease;
	}

	.alert-close:hover {
		background: rgba(255, 255, 255, 0.2);
		color: #374151;
		transform: scale(1.1);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}

	.alert-close:hover::before {
		opacity: 1;
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
		margin-bottom: 0.5rem;
	}

	.auto-close-countdown {
		font-size: 0.8rem;
		color: #6b7280;
		text-align: center;
		padding: 0.5rem;
		background: rgba(59, 130, 246, 0.1);
		border-radius: 6px;
		border: 1px solid rgba(59, 130, 246, 0.2);
		animation: pulse 1s infinite;
	}

	@keyframes pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.7;
		}
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
		background: linear-gradient(135deg, 
			rgba(59, 130, 246, 0.9) 0%, 
			rgba(37, 99, 235, 0.9) 100%);
		backdrop-filter: blur(10px);
		color: white;
		box-shadow: 
			0 4px 12px rgba(59, 130, 246, 0.3),
			inset 0 1px 0 rgba(255, 255, 255, 0.2);
		border: 1px solid rgba(255, 255, 255, 0.2);
		position: relative;
		overflow: hidden;
	}

	.alert-btn-primary::before {
		content: '';
		position: absolute;
		top: 0;
		left: -100%;
		width: 100%;
		height: 100%;
		background: linear-gradient(90deg, 
			transparent, 
			rgba(255, 255, 255, 0.2), 
			transparent);
		transition: left 0.5s;
	}

	.alert-btn-primary:hover {
		background: linear-gradient(135deg, 
			rgba(59, 130, 246, 1) 0%, 
			rgba(37, 99, 235, 1) 100%);
		transform: translateY(-2px) scale(1.02);
		box-shadow: 
			0 8px 20px rgba(59, 130, 246, 0.4),
			0 4px 12px rgba(59, 130, 246, 0.2),
			inset 0 1px 0 rgba(255, 255, 255, 0.3);
	}

	.alert-btn-primary:hover::before {
		left: 100%;
	}

	.alert-btn-primary:active {
		transform: translateY(-1px) scale(1.01);
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background: transparent;
		overflow: hidden;
		/* 핵심: 전체 body는 클릭 이벤트를 무시 */
		/* pointer-events: none; */
	}

	:global(html) {
		background: transparent;
		height: 100%;
	}
</style>

