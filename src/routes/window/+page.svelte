<script lang="ts">
	import Panel from '../panel/+page.svelte';

	type Tab = {
		id: number;
		name: string;
		active: boolean;
		selected: boolean;
		windowId: number;
	};

	type Window = {
		id: number;
		position: { x: number; y: number };
		size: { width: number; height: number };
		tabs: Tab[];
		zIndex: number;
	};

	// Global state
	let windows = $state<Window[]>([
		{
			id: 1,
			position: { x: 0, y: 0 },
			size: { width: 560, height: 600 },
			tabs: [
				{ id: 1, name: 'TAG NAME 1', active: true, selected: false, windowId: 1 },
				{ id: 2, name: 'TAG 2', active: false, selected: false, windowId: 1 }
			],
			zIndex: 1
		}
	]);
	
	let nextTabId = 3;
	let nextWindowId = 2;
	let maxZIndex = 1;

	// Zoom & Pan
	let zoom = $state(1);
	let panOffset = $state({ x: 0, y: 0 });
	let isPanning = $state(false);
	let panStart = { x: 0, y: 0 };
	const MIN_ZOOM = 0.5;
	const MAX_ZOOM = 2;
	const ZOOM_STEP = 0.1;

	// Snap
	let snapEnabled = $state(true);
	const GRID_SIZE = 20;

	// Window dragging
	let draggingWindowId = $state<number | null>(null);
	let dragOffset = $state({ x: 0, y: 0 });

	// Window resizing
	let resizingWindowId = $state<number | null>(null);
	let resizeDirection = $state('');
	let resizeStart = $state({ x: 0, y: 0, width: 0, height: 0 });
	let showSizeTooltip = $state(false);
	let resizeWindowSize = $state({ width: 0, height: 0 });
	const MIN_WIDTH = 480;
	const MIN_HEIGHT = 600;

	// Tab dragging
	let draggingTab = $state<{ tab: Tab; windowId: number } | null>(null);
	let dragTabOffset = $state({ x: 0, y: 0 });
	let dragTabPosition = $state({ x: 0, y: 0 });
	let showDragPreview = $state(false);

	// Tab reordering
	let reorderingTab = $state<{ tab: Tab; windowId: number; startIndex: number } | null>(null);
	let dragOverTabIndex = $state<number | null>(null);

	// Multi-select
	let lastClickedTabIndex = $state<number | null>(null);
	let shiftKeyPressed = $state(false);
	let ctrlKeyPressed = $state(false);

	// Window manager panel
	let showWindowManager = $state(true);

	function snapToGrid(value: number): number {
		return Math.round(value / GRID_SIZE) * GRID_SIZE;
	}

	function bringWindowToFront(windowId: number) {
		maxZIndex++;
		windows = windows.map(w => 
			w.id === windowId ? { ...w, zIndex: maxZIndex } : w
		);
	}

	function getWindow(windowId: number): Window | undefined {
		return windows.find(w => w.id === windowId);
	}

	function updateWindow(windowId: number, updates: Partial<Window>) {
		windows = windows.map(w => 
			w.id === windowId ? { ...w, ...updates } : w
		);
	}

	function addTab(windowId: number) {
		const window = getWindow(windowId);
		if (!window) return;

		const newTab: Tab = {
			id: nextTabId++,
			name: `TAG ${nextTabId - 1}`,
			active: true,
			selected: false,
			windowId
		};

		const updatedTabs = window.tabs.map(t => ({ ...t, active: false }));
		updateWindow(windowId, { tabs: [...updatedTabs, newTab] });
	}

	function setActiveTab(windowId: number, tabId: number) {
		const window = getWindow(windowId);
		if (!window) return;

		const updatedTabs = window.tabs.map(t => ({
			...t,
			active: t.id === tabId
		}));
		updateWindow(windowId, { tabs: updatedTabs });
	}

	function closeTab(windowId: number, tabId: number) {
		const window = getWindow(windowId);
		if (!window) return;

		const index = window.tabs.findIndex(t => t.id === tabId);
		if (index === -1) return;

		const wasActive = window.tabs[index].active;
		const updatedTabs = window.tabs.filter(t => t.id !== tabId);

		if (updatedTabs.length === 0) {
			// Close window if no tabs left
			windows = windows.filter(w => w.id !== windowId);
			return;
		}

		if (wasActive) {
			const newIndex = Math.min(index, updatedTabs.length - 1);
			updatedTabs[newIndex].active = true;
		}

		updateWindow(windowId, { tabs: updatedTabs });
	}

	function handleTitleBarMouseDown(e: MouseEvent, windowId: number) {
		if ((e.target as HTMLElement).closest('.tab') || 
		    (e.target as HTMLElement).closest('.tab-add-btn') ||
		    (e.target as HTMLElement).closest('.window-controls') ||
		    (e.target as HTMLElement).closest('.resize-handle')) {
			return;
		}

		e.preventDefault();
		const window = getWindow(windowId);
		if (!window) return;

		draggingWindowId = windowId;
		dragOffset = {
			x: e.clientX / zoom - panOffset.x - window.position.x,
			y: e.clientY / zoom - panOffset.y - window.position.y
		};
		bringWindowToFront(windowId);
	}

	function handleResizeStart(e: MouseEvent, windowId: number, direction: string) {
		e.preventDefault();
		e.stopPropagation();
		const window = getWindow(windowId);
		if (!window) return;

		resizingWindowId = windowId;
		resizeDirection = direction;
		resizeStart = {
			x: e.clientX / zoom - panOffset.x,
			y: e.clientY / zoom - panOffset.y,
			width: window.size.width,
			height: window.size.height
		};
		resizeWindowSize = { ...window.size };
		showSizeTooltip = true;
		bringWindowToFront(windowId);
	}

	function handleTabMouseDown(e: MouseEvent, windowId: number, tab: Tab, index: number) {
		const window = getWindow(windowId);
		if (!window) return;

		// Handle multi-select
		if (e.shiftKey && lastClickedTabIndex !== null) {
			e.preventDefault();
			const start = Math.min(lastClickedTabIndex, index);
			const end = Math.max(lastClickedTabIndex, index);
			const updatedTabs = window.tabs.map((t, i) => ({
				...t,
				selected: i >= start && i <= end
			}));
			updateWindow(windowId, { tabs: updatedTabs });
			return;
		}

		if (e.ctrlKey) {
			e.preventDefault();
			const updatedTabs = window.tabs.map(t =>
				t.id === tab.id ? { ...t, selected: !t.selected } : t
			);
			updateWindow(windowId, { tabs: updatedTabs });
			lastClickedTabIndex = index;
			return;
		}

		lastClickedTabIndex = index;

		// Start dragging tab for detachment
		draggingTab = { tab, windowId };
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		dragTabOffset = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top
		};
		dragTabPosition = {
			x: e.clientX,
			y: e.clientY
		};
	}

	function handleTabDragForReorder(e: MouseEvent, windowId: number, tab: Tab, index: number) {
		if (e.button !== 0) return; // Only left click
		
		const target = e.target as HTMLElement;
		if (target.closest('.tab-close') || target.closest('.tab-add-btn')) {
			return;
		}

		reorderingTab = { tab, windowId, startIndex: index };
	}

	function handleMouseMove(e: MouseEvent) {
		// Window dragging
		if (draggingWindowId !== null) {
			const window = getWindow(draggingWindowId);
			if (window) {
				let newX = e.clientX / zoom - panOffset.x - dragOffset.x;
				let newY = e.clientY / zoom - panOffset.y - dragOffset.y;

				if (snapEnabled) {
					newX = snapToGrid(newX);
					newY = snapToGrid(newY);
				}

				updateWindow(draggingWindowId, { position: { x: newX, y: newY } });
			}
		}

		// Window resizing
		if (resizingWindowId !== null) {
			const window = getWindow(resizingWindowId);
			if (window) {
				const currentX = e.clientX / zoom - panOffset.x;
				const currentY = e.clientY / zoom - panOffset.y;
				const dx = currentX - resizeStart.x;
				const dy = currentY - resizeStart.y;

				let newWidth = window.size.width;
				let newHeight = window.size.height;
				let newX = window.position.x;
				let newY = window.position.y;

				if (resizeDirection.includes('e')) {
					newWidth = Math.max(MIN_WIDTH, resizeStart.width + dx);
				}
				if (resizeDirection.includes('w')) {
					const widthDiff = Math.min(dx, resizeStart.width - MIN_WIDTH);
					newWidth = resizeStart.width - widthDiff;
					newX = window.position.x + widthDiff;
				}
				if (resizeDirection.includes('s')) {
					newHeight = Math.max(MIN_HEIGHT, resizeStart.height + dy);
				}
				if (resizeDirection.includes('n')) {
					const heightDiff = Math.min(dy, resizeStart.height - MIN_HEIGHT);
					newHeight = resizeStart.height - heightDiff;
					newY = window.position.y + heightDiff;
				}

				resizeWindowSize = { width: newWidth, height: newHeight };
				updateWindow(resizingWindowId, {
					size: { width: newWidth, height: newHeight },
					position: { x: newX, y: newY }
				});
			}
		}

		// Tab dragging for detachment
		if (draggingTab) {
			dragTabPosition = { x: e.clientX, y: e.clientY };
			showDragPreview = true;

			// Check if dragging outside the original window
			const sourceWindow = getWindow(draggingTab.windowId);
			if (sourceWindow) {
				const distance = Math.sqrt(
					Math.pow(e.clientX - dragTabPosition.x, 2) +
					Math.pow(e.clientY - dragTabPosition.y, 2)
				);
				// Show preview if dragged more than 10px
				showDragPreview = distance > 10;
			}
		}

		// Pan
		if (isPanning) {
			panOffset = {
				x: panOffset.x + (e.clientX - panStart.x) / zoom,
				y: panOffset.y + (e.clientY - panStart.y) / zoom
			};
			panStart = { x: e.clientX, y: e.clientY };
		}

		// Tab reordering
		if (reorderingTab) {
			// Find which tab is being hovered over
			const tabElements = document.querySelectorAll(`[data-window-id="${reorderingTab.windowId}"] .tab`);
			let hoveredIndex: number | null = null;
			
			tabElements.forEach((el, index) => {
				const rect = el.getBoundingClientRect();
				if (e.clientX >= rect.left && e.clientX <= rect.right &&
				    e.clientY >= rect.top && e.clientY <= rect.bottom) {
					hoveredIndex = index;
				}
			});
			
			dragOverTabIndex = hoveredIndex;
		}
	}

	function handleMouseUp(e: MouseEvent) {
		// Tab detachment
		if (draggingTab && showDragPreview) {
			const { tab, windowId: sourceWindowId } = draggingTab;
			
			// Check if dropped on another window
			let droppedOnWindowId: number | null = null;
			
			for (const window of windows) {
				const windowEl = document.querySelector(`[data-window-id="${window.id}"]`);
				if (windowEl) {
					const rect = windowEl.getBoundingClientRect();
					if (e.clientX >= rect.left && e.clientX <= rect.right &&
					    e.clientY >= rect.top && e.clientY <= rect.bottom) {
						droppedOnWindowId = window.id;
						break;
					}
				}
			}

			if (droppedOnWindowId && droppedOnWindowId !== sourceWindowId) {
				// Merge tab into another window
				const sourceWindow = getWindow(sourceWindowId);
				const targetWindow = getWindow(droppedOnWindowId);
				
				if (sourceWindow && targetWindow) {
					const tabToMove = sourceWindow.tabs.find(t => t.id === tab.id);
					if (tabToMove) {
						// Remove from source
						const updatedSourceTabs = sourceWindow.tabs.filter(t => t.id !== tab.id);
						
						if (updatedSourceTabs.length === 0) {
							// Close source window if no tabs left
							windows = windows.filter(w => w.id !== sourceWindowId);
						} else {
							// Set a new active tab if needed
							if (tabToMove.active && updatedSourceTabs.length > 0) {
								updatedSourceTabs[0].active = true;
							}
							updateWindow(sourceWindowId, { tabs: updatedSourceTabs });
						}
						
						// Add to target
						const updatedTargetTabs = targetWindow.tabs.map(t => ({ ...t, active: false }));
						updatedTargetTabs.push({ ...tabToMove, active: true, windowId: droppedOnWindowId });
						updateWindow(droppedOnWindowId, { tabs: updatedTargetTabs });
					}
				}
			} else if (!droppedOnWindowId) {
				// Create new window
				const sourceWindow = getWindow(sourceWindowId);
				if (sourceWindow && sourceWindow.tabs.length > 1) {
					const tabToDetach = sourceWindow.tabs.find(t => t.id === tab.id);
					if (tabToDetach) {
						// Remove from source
						const updatedSourceTabs = sourceWindow.tabs.filter(t => t.id !== tab.id);
						if (tabToDetach.active && updatedSourceTabs.length > 0) {
							updatedSourceTabs[0].active = true;
						}
						updateWindow(sourceWindowId, { tabs: updatedSourceTabs });
						
						// Create new window
						const newWindow: Window = {
							id: nextWindowId++,
							position: {
								x: (e.clientX / zoom - panOffset.x) - 280,
								y: (e.clientY / zoom - panOffset.y) - 20
							},
							size: { width: 560, height: 600 },
							tabs: [{ ...tabToDetach, active: true, windowId: nextWindowId - 1 }],
							zIndex: ++maxZIndex
						};
						windows = [...windows, newWindow];
					}
				}
			}
		}

		// Tab reordering
		if (reorderingTab && dragOverTabIndex !== null && dragOverTabIndex !== reorderingTab.startIndex) {
			const window = getWindow(reorderingTab.windowId);
			if (window) {
				const newTabs = [...window.tabs];
				const [movedTab] = newTabs.splice(reorderingTab.startIndex, 1);
				newTabs.splice(dragOverTabIndex, 0, movedTab);
				updateWindow(reorderingTab.windowId, { tabs: newTabs });
			}
		}

		draggingWindowId = null;
		resizingWindowId = null;
		showSizeTooltip = false;
		draggingTab = null;
		showDragPreview = false;
		isPanning = false;
		reorderingTab = null;
		dragOverTabIndex = null;
	}

	function handleWheel(e: WheelEvent) {
		if (e.ctrlKey) {
			e.preventDefault();
			const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
			zoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom + delta));
		}
	}

	function handleMouseDownForPan(e: MouseEvent) {
		if (e.button === 1) { // Middle mouse button
			e.preventDefault();
			isPanning = true;
			panStart = { x: e.clientX, y: e.clientY };
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.ctrlKey) {
			ctrlKeyPressed = true;
			if (e.key === '-' || e.key === '_') {
				e.preventDefault();
				zoom = Math.max(MIN_ZOOM, zoom - ZOOM_STEP);
			} else if (e.key === '=' || e.key === '+') {
				e.preventDefault();
				zoom = Math.min(MAX_ZOOM, zoom + ZOOM_STEP);
			} else if (e.key === '0') {
				e.preventDefault();
				zoom = 1;
			}
		}

		if (e.shiftKey) {
			shiftKeyPressed = true;
		}

		if (e.key === 's' || e.key === 'S') {
			if (!e.ctrlKey && !e.altKey && !e.metaKey) {
				snapEnabled = !snapEnabled;
			}
		}
	}

	function handleKeyUp(e: KeyboardEvent) {
		if (!e.ctrlKey) {
			ctrlKeyPressed = false;
		}
		if (!e.shiftKey) {
			shiftKeyPressed = false;
		}
	}

	function centerWindow(windowId: number) {
		const window = getWindow(windowId);
		if (!window) return;

		updateWindow(windowId, {
			position: { x: 0, y: 0 }
		});
		bringWindowToFront(windowId);
	}

	function isWindowOffScreen(window: Window): boolean {
		const viewWidth = (typeof globalThis.innerWidth !== 'undefined' ? globalThis.innerWidth : 1920) / zoom;
		const viewHeight = (typeof globalThis.innerHeight !== 'undefined' ? globalThis.innerHeight : 1080) / zoom;
		
		const windowCenterX = window.position.x + panOffset.x + window.size.width / 2;
		const windowCenterY = window.position.y + panOffset.y + window.size.height / 2;
		
		return windowCenterX < -viewWidth / 2 || windowCenterX > viewWidth * 1.5 ||
		       windowCenterY < -viewHeight / 2 || windowCenterY > viewHeight * 1.5;
	}

	function closeWindow(windowId: number) {
		windows = windows.filter(w => w.id !== windowId);
	}

	function createNewWindow() {
		const newWindow: Window = {
			id: nextWindowId++,
			position: { x: 50 + (windows.length * 30), y: 50 + (windows.length * 30) },
			size: { width: 560, height: 600 },
			tabs: [{
				id: nextTabId++,
				name: `TAG ${nextTabId - 1}`,
				active: true,
				selected: false,
				windowId: nextWindowId - 1
			}],
			zIndex: ++maxZIndex
		};
		windows = [...windows, newWindow];
	}
</script>

<svelte:window 
	onmousemove={handleMouseMove} 
	onmouseup={handleMouseUp}
	onwheel={handleWheel}
	onkeydown={handleKeyDown}
	onkeyup={handleKeyUp}
	onmousedown={handleMouseDownForPan}
/>

<div class="viewport" style="transform: scale({zoom});">
	<div class="canvas" style="transform: translate({panOffset.x}px, {panOffset.y}px);">
		{#each windows as window (window.id)}
			<div 
				class="window"
				data-window-id={window.id}
				style="
					transform: translate({window.position.x}px, {window.position.y}px);
					width: {window.size.width}px;
					height: {window.size.height}px;
					z-index: {window.zIndex};
				"
			>
				<div 
					class="title-bar" 
					role="toolbar"
					tabindex="-1"
					onmousedown={(e) => handleTitleBarMouseDown(e, window.id)}
				>
					<div class="tabs">
						{#each window.tabs as tab, index (tab.id)}
							<div 
								class="tab"
								class:active={tab.active}
								class:selected={tab.selected}
								class:drag-over={reorderingTab?.windowId === window.id && dragOverTabIndex === index}
								onclick={() => setActiveTab(window.id, tab.id)}
								onmousedown={(e) => {
									handleTabMouseDown(e, window.id, tab, index);
									handleTabDragForReorder(e, window.id, tab, index);
								}}
								onkeydown={(e) => e.key === 'Enter' && setActiveTab(window.id, tab.id)}
								role="tab"
								tabindex="0"
								aria-selected={tab.active}
							>
								<span class="tab-icon">🎨</span>
								<span class="tab-name">{tab.name}</span>
								<button 
									class="tab-close"
									onclick={(e) => { e.stopPropagation(); closeTab(window.id, tab.id); }}
									aria-label="Close tab"
								>
									✕
								</button>
							</div>
						{/each}
						<button 
							class="tab-add-btn"
							onclick={() => addTab(window.id)}
							aria-label="Add tab"
						>
							+
						</button>
					</div>

					<div class="window-controls">
						<button 
							class="control-btn minimize"
							onclick={() => centerWindow(window.id)}
							aria-label="Center"
							title="Center window"
						>
							○
						</button>
						<button 
							class="control-btn close"
							onclick={() => closeWindow(window.id)}
							aria-label="Close"
						>
							✕
						</button>
					</div>
				</div>

				<div class="content">
					{#each window.tabs as tab}
						{#if tab.active}
							<div class="tab-content">
								<Panel />
							</div>
						{/if}
					{/each}
				</div>
				
				<!-- Resize Handles -->
				<div class="resize-handle resize-n" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'n')}></div>
				<div class="resize-handle resize-s" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 's')}></div>
				<div class="resize-handle resize-e" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'e')}></div>
				<div class="resize-handle resize-w" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'w')}></div>
				<div class="resize-handle resize-ne" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'ne')}></div>
				<div class="resize-handle resize-nw" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'nw')}></div>
				<div class="resize-handle resize-se" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'se')}></div>
				<div class="resize-handle resize-sw" role="presentation" onmousedown={(e) => handleResizeStart(e, window.id, 'sw')}></div>
				
				<!-- Size Tooltip -->
				{#if showSizeTooltip && resizingWindowId === window.id}
					<div class="size-tooltip">
						{Math.round(resizeWindowSize.width)} × {Math.round(resizeWindowSize.height)} px
					</div>
				{/if}
			</div>
		{/each}

		<!-- Drag Preview -->
		{#if showDragPreview && draggingTab}
			<div 
				class="tab-drag-preview"
				style="
					left: {dragTabPosition.x - dragTabOffset.x}px;
					top: {dragTabPosition.y - dragTabOffset.y}px;
				"
			>
				<span class="tab-icon">🎨</span>
				<span class="tab-name">{draggingTab.tab.name}</span>
			</div>
		{/if}
	</div>
</div>

<!-- Control Panel -->
<div class="control-panel">
	<div class="control-item">
		<span class="control-label">Zoom: {(zoom * 100).toFixed(0)}%</span>
		<div class="control-buttons">
			<button onclick={() => zoom = Math.max(MIN_ZOOM, zoom - ZOOM_STEP)}>−</button>
			<button onclick={() => zoom = 1}>Reset</button>
			<button onclick={() => zoom = Math.min(MAX_ZOOM, zoom + ZOOM_STEP)}>+</button>
		</div>
	</div>
	<div class="control-item">
		<label class="control-label">
			<input type="checkbox" bind:checked={snapEnabled} />
			Snap to Grid
		</label>
	</div>
	<button class="action-btn" onclick={createNewWindow}>
		+ New Window
	</button>
	<button class="action-btn" onclick={() => showWindowManager = !showWindowManager}>
		{showWindowManager ? 'Hide' : 'Show'} Windows
	</button>
</div>

<!-- Window Manager -->
{#if showWindowManager}
	<div class="window-manager">
		<div class="manager-header">
			<h3>Windows ({windows.length})</h3>
			<button onclick={() => showWindowManager = false}>✕</button>
		</div>
		<div class="manager-list">
			{#each windows as window (window.id)}
				<div 
					class="manager-item"
					class:off-screen={isWindowOffScreen(window)}
					role="button"
					tabindex="0"
					onclick={() => {
						centerWindow(window.id);
						bringWindowToFront(window.id);
					}}
					onkeydown={(e) => {
						if (e.key === 'Enter' || e.key === ' ') {
							e.preventDefault();
							centerWindow(window.id);
							bringWindowToFront(window.id);
						}
					}}
				>
					<div class="manager-item-info">
						<strong>Window {window.id}</strong>
						<span>{window.tabs.length} tabs</span>
						{#if isWindowOffScreen(window)}
							<span class="warning">⚠ Off-screen</span>
						{/if}
					</div>
					<button 
						class="manager-item-close"
						onclick={(e) => { e.stopPropagation(); closeWindow(window.id); }}
					>
						✕
					</button>
				</div>
			{/each}
		</div>
	</div>
{/if}

<!-- Instructions -->
<div class="instructions">
	<div class="instruction-item">Ctrl + Wheel: Zoom</div>
	<div class="instruction-item">Middle Click: Pan</div>
	<div class="instruction-item">Shift: Range Select</div>
	<div class="instruction-item">Ctrl: Multi Select</div>
	<div class="instruction-item">S: Toggle Snap</div>
</div>

<style>
	:global(body) {
		margin: 0;
		padding: 0;
		overflow: hidden;
	}

	.viewport {
		width: 100vw;
		height: 100vh;
		position: relative;
		transform-origin: center center;
		will-change: transform;
		background: var(--bg-primary);
		transition: background-color 0.3s ease;
	}

	.canvas {
		width: 100%;
		height: 100%;
		position: relative;
		will-change: transform;
		background-image: radial-gradient(circle, var(--text-muted) 1px, transparent 1px);
		background-size: 20px 20px;
		background-position: 0 0;
	}

	.window {
		background: var(--bg-secondary);
		border-radius: 8px;
		box-shadow: 0 8px 32px var(--shadow);
		position: absolute;
		left: 50%;
		top: 50%;
		display: flex;
		flex-direction: column;
		will-change: transform;
		border: 2px solid var(--border-color);
		transition: all 0.2s ease;
	}

	.title-bar {
		background: var(--bg-tertiary);
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0;
		user-select: none;
		cursor: move;
		border-bottom: 1px solid var(--border-color);
		flex-shrink: 0;
		transition: all 0.3s ease;
	}

	.tabs {
		display: flex;
		gap: 2px;
		padding: 4px 4px 0 4px;
		flex: 1;
		overflow-x: auto;
		overflow-y: hidden;
		scrollbar-width: none;
		align-items: flex-end;
	}

	.tabs::-webkit-scrollbar {
		display: none;
	}

	.tab {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		background: var(--bg-secondary);
		border-radius: 6px 6px 0 0;
		cursor: pointer;
		transition: background-color 0.15s ease;
		white-space: nowrap;
		min-width: 120px;
		border: 1px solid transparent;
		font-family: inherit;
		font-size: inherit;
		position: relative;
	}

	.tab:hover {
		background: var(--bg-tertiary);
	}

	.tab.active {
		background: var(--bg-secondary);
		border-color: var(--border-color);
		border-bottom-color: transparent;
	}

	.tab.selected {
		background: var(--btn-primary);
		border-color: var(--btn-primary-hover);
	}

	.tab.selected .tab-name {
		color: white;
	}

	.tab.drag-over {
		border-left: 3px solid var(--btn-primary);
	}

	.tab-icon {
		font-size: 14px;
		flex-shrink: 0;
	}

	.tab-name {
		color: var(--text-primary);
		font-size: 13px;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
		flex: 1;
	}

	.tab-close {
		background: transparent;
		border: none;
		color: var(--text-muted);
		font-size: 16px;
		cursor: pointer;
		padding: 0;
		width: 18px;
		height: 18px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 3px;
		flex-shrink: 0;
		transition: all 0.15s ease;
	}

	.tab-close:hover {
		background: #ff5555;
		color: white;
	}

	.tab-add-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		font-size: 18px;
		cursor: pointer;
		padding: 4px 12px;
		margin-bottom: 0;
		border-radius: 6px 6px 0 0;
		transition: all 0.15s ease;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.tab-add-btn:hover {
		background: var(--bg-tertiary);
		color: var(--text-primary);
	}

	.window-controls {
		display: flex;
		gap: 0;
		padding-right: 4px;
	}

	.control-btn {
		width: 46px;
		height: 32px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		font-size: 14px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background-color 0.15s ease;
	}

	.control-btn:hover {
		background: var(--bg-tertiary);
	}

	.control-btn.close:hover {
		background: #e81123;
		color: white;
	}

	.control-btn.minimize {
		font-size: 16px;
	}

	.content {
		padding: 0;
		background: var(--bg-secondary);
		color: var(--text-primary);
		flex: 1;
		overflow: auto;
		transition: all 0.3s ease;
	}

	.tab-content {
		animation: fadeIn 0.2s ease;
		height: 100%;
		width: 100%;
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	/* Resize Handles */
	.resize-handle {
		position: absolute;
		z-index: 10;
	}

	.resize-n, .resize-s {
		height: 8px;
		left: 8px;
		right: 8px;
		cursor: ns-resize;
	}

	.resize-n { top: 0; }
	.resize-s { bottom: 0; }

	.resize-e, .resize-w {
		width: 8px;
		top: 8px;
		bottom: 8px;
		cursor: ew-resize;
	}

	.resize-e { right: 0; }
	.resize-w { left: 0; }

	.resize-ne, .resize-nw, .resize-se, .resize-sw {
		width: 12px;
		height: 12px;
	}

	.resize-ne { top: 0; right: 0; cursor: nesw-resize; }
	.resize-nw { top: 0; left: 0; cursor: nwse-resize; }
	.resize-se { bottom: 0; right: 0; cursor: nwse-resize; }
	.resize-sw { bottom: 0; left: 0; cursor: nesw-resize; }

	.size-tooltip {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		background: rgba(0, 0, 0, 0.9);
		color: white;
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		pointer-events: none;
		z-index: 1000;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
	}

	.tab-drag-preview {
		position: fixed;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		background: var(--btn-primary);
		border-radius: 6px;
		pointer-events: none;
		z-index: 10000;
		box-shadow: 0 4px 16px var(--shadow);
		color: white;
		font-size: 13px;
	}

	.control-panel {
		position: fixed;
		bottom: 24px;
		right: 24px;
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 12px;
		box-shadow: 0 4px 16px var(--shadow);
		min-width: 200px;
		z-index: 10000;
		transition: all 0.3s ease;
	}

	.control-item {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.control-label {
		color: var(--text-primary);
		font-size: 13px;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.control-label input[type="checkbox"] {
		width: 16px;
		height: 16px;
		cursor: pointer;
	}

	.control-buttons {
		display: flex;
		gap: 6px;
	}

	.control-buttons button {
		flex: 1;
		padding: 6px 12px;
		background: var(--bg-tertiary);
		color: var(--text-primary);
		border: 1px solid var(--border-color);
		border-radius: 4px;
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.control-buttons button:hover {
		background: var(--btn-secondary);
		color: white;
	}

	.action-btn {
		padding: 10px 20px;
		background: var(--btn-success);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		width: 100%;
	}

	.action-btn:hover {
		background: var(--btn-success-hover);
		transform: scale(1.02);
	}

	.window-manager {
		position: fixed;
		top: 24px;
		right: 24px;
		background: var(--bg-secondary);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		box-shadow: 0 4px 16px var(--shadow);
		min-width: 280px;
		max-width: 320px;
		max-height: 400px;
		display: flex;
		flex-direction: column;
		z-index: 10000;
		transition: all 0.3s ease;
	}

	.manager-header {
		padding: 12px 16px;
		border-bottom: 1px solid var(--border-color);
		display: flex;
		justify-content: space-between;
		align-items: center;
		transition: all 0.3s ease;
	}

	.manager-header h3 {
		margin: 0;
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.manager-header button {
		background: transparent;
		border: none;
		color: var(--text-muted);
		font-size: 18px;
		cursor: pointer;
		padding: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		transition: all 0.15s ease;
	}

	.manager-header button:hover {
		background: var(--bg-tertiary);
		color: var(--text-primary);
	}

	.manager-list {
		overflow-y: auto;
		flex: 1;
	}

	.manager-item {
		padding: 12px 16px;
		border-bottom: 1px solid #3a3a3a;
		cursor: pointer;
		display: flex;
		justify-content: space-between;
		align-items: center;
		transition: background-color 0.15s ease;
	}

	.manager-item:hover {
		background: #333;
	}

	.manager-item.off-screen {
		background: #3a2a2a;
	}

	.manager-item-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.manager-item-info strong {
		color: #e0e0e0;
		font-size: 13px;
	}

	.manager-item-info span {
		color: #888;
		font-size: 12px;
	}

	.manager-item-info .warning {
		color: #ff9800;
	}

	.manager-item-close {
		background: transparent;
		border: none;
		color: #888;
		font-size: 16px;
		cursor: pointer;
		padding: 4px;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
	}

	.manager-item-close:hover {
		background: #ff5555;
		color: white;
	}

	.instructions {
		position: fixed;
		bottom: 24px;
		left: 24px;
		background: rgba(42, 42, 42, 0.95);
		border: 1px solid #3a3a3a;
		border-radius: 8px;
		padding: 12px 16px;
		display: flex;
		flex-direction: column;
		gap: 6px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
		z-index: 10000;
	}

	.instruction-item {
		color: #aaa;
		font-size: 12px;
		font-family: monospace;
	}
</style>
