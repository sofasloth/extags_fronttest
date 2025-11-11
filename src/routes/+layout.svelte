<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../css/global.css';
	import { beforeNavigate, afterNavigate } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import {getCurrentWindow} from '@tauri-apps/api/window';
	import {page} from '$app/stores';

	const routesToHideLayout = ['/alert'];

	let { children } = $props();
	
	let isDarkMode = $state(true);
	let isNavigating = $state(false);
	let hideLayout = $state(false);
	let appWindow: any = null;
	
	function toggleTheme() {
		isDarkMode = !isDarkMode;
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', isDarkMode ? 'dark' : 'light');
		}
	}
	
	// Initialize theme on mount
	$effect(() => {
		if (typeof document !== 'undefined') {
			document.documentElement.setAttribute('data-theme', isDarkMode ? 'dark' : 'light');
		}
	});

	// Tauri API 초기화 (브라우저 환경에서만)
	$effect(() => {
		if (typeof window !== 'undefined') {
			const tauriWindow = window as any;
			console.log('Checking for Tauri:', tauriWindow.__TAURI__);
			if (tauriWindow.__TAURI__) {
				console.log('Tauri found, initializing appWindow');
				appWindow = getCurrentWindow();
				console.log('appWindow initialized:', appWindow);
			} else {
				console.log('Tauri not found');
			}
		}

		hideLayout = routesToHideLayout.includes($page.url.pathname);
	});

	// 하위 컴포넌트에서 사용할 수 있도록 window에 노출
	$effect(() => {
		if (typeof window !== 'undefined') {
			(window as any).__toggleTheme = toggleTheme;
			(window as any).__isDarkMode = isDarkMode;
		}
	});

	beforeNavigate(() => {
		isNavigating = true;
	});

	afterNavigate(() => {
		isNavigating = false;
	});

	async function minimizeWindow() {
		appWindow = await getCurrentWindow();
		console.log('minimizeWindow called');
		console.log('appWindow:', appWindow);
		if (appWindow) {
			console.log('Calling appWindow.minimize()');
			await appWindow.minimize();
		} else {
			console.log('appWindow is null, cannot minimize');
		}
	}

	async function maximizeWindow() {
		appWindow = await getCurrentWindow();
		console.log('maximizeWindow called');
		console.log('appWindow:', appWindow);
		if (appWindow) {
			console.log('Calling appWindow.toggleMaximize()');
			await appWindow.toggleMaximize();
		} else {
			console.log('appWindow is null, cannot maximize');
		}
	}

	async function closeWindow() {
		appWindow = await getCurrentWindow();
		console.log('closeWindow called');
		console.log('appWindow:', appWindow);
		if (appWindow) {
			console.log('Calling appWindow.close()');
			await appWindow.close();
		} else {
			console.log('appWindow is null, cannot close');
		}
	}
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if !hideLayout}
<nav data-tauri-drag-region>
	<div class="nav-links">
		<a href="/" class="nav-link">Home</a>
		<a href="/memo" class="nav-link">Memo</a>
		<a href="/window" class="nav-link">window</a>
		<a href="/file_interaction" class="nav-link">File Interaction</a>
		<a href="/search_result" class="nav-link">Search Result</a>
		<a href="/search_window" class="nav-link">Search Window</a>
		<a href="/new_searchbar">Search Bar</a>
	</div>
	<div class="flex-spacer" data-tauri-drag-region></div>
	<button class="theme-toggle" onclick={toggleTheme} title="Toggle theme">
		{isDarkMode ? '☀️' : '🌙'}
	</button>
	<button id="titlebar-minimize" title="Minimize" class="titlebar-button" onclick={minimizeWindow}>
		<svg
        xmlns="http://www.w3.org/2000/svg"
        width="12"
        height="12"
        viewBox="0 0 24 24"
      >
        <path fill="currentColor" d="M19 13H5v-2h14z" />
		</svg>
	</button>
	<button id="titlebar-maximize" title="Maximize" class="titlebar-button" onclick={maximizeWindow}>
		<svg
        xmlns="http://www.w3.org/2000/svg"
        width="12"
        height="12"
        viewBox="0 0 24 24"
      >
        <path fill="currentColor" d="M4 4h16v16H4zm2 4v10h12V8z" />
		</svg>
	</button>
	<button id="titlebar-close" title="Close" class="titlebar-button" onclick={closeWindow}>
		<svg
        xmlns="http://www.w3.org/2000/svg"
        width="12"
        height="12"
        viewBox="0 0 24 24"
      >
        <path
          fill="currentColor"
          d="M13.46 12L19 17.54V19h-1.46L12 13.46L6.46 19H5v-1.46L10.54 12L5 6.46V5h1.46L12 10.54L17.54 5H19v1.46z"
        />
      </svg>
	</button>
</nav>
{/if}

{#if isNavigating}
	<div class="loading-overlay" transition:fade={{duration: 200}}>
		<div class="spinner">
			<p>Loading...</p>
		</div>
	</div>
{/if}

{@render children?.()}
 
<style>
	:global(body) {
		overflow: hidden;
	}

	nav {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.2rem 0.4rem;
		background: var(--bg-primary);
		border-bottom: 1px solid var(--border-color);
		gap: 1rem;
		margin-bottom: 1rem;
		font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
		font-size: 1.0rem;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 1000;
	}

	nav[data-tauri-drag-region] {
		app-region: drag;
	}

	/* nav:not(:hover) {
		opacity: 0;
		transition: opacity 0.2s ease;
		pointer-events: visible;
	} */
	
	.nav-links {
		display: flex;
		gap: 0.5rem;
	}
	
	nav a {
		color: var(--text-primary);
		text-decoration: none;
		padding: 0 0.2rem;
		margin: 0 0.2rem;
		border-radius: 4px;
		transition: background-color 0.2s ease;
		user-select: none;
	}
	
	nav a:hover {
		background: var(--bg-tertiary);
	}
	
	.theme-toggle {
		padding: 0 0.2rem;
		background: var(--btn-primary);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 1.2rem;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 48px;
	}

	.flex-spacer {
		flex: 1;
		height: 100%;
	}

	.titlebar-button {
		padding: 0 0.4rem;
		background-color: transparent;
		color: var(--text-primary);
		border: none;
		border-radius: 6px;
		font-size: 1.2rem;
		cursor: pointer;
	}

	.titlebar-button:hover {
		background-color: var(--bg-tertiary);
		transition: background-color 0.2s ease;
	}
	
	.theme-toggle:hover {
		background: var(--btn-primary-hover);
		transform: scale(1.05);
	}

	.loading-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  p {
    margin-top: 20px;
    color: white;
  }
</style>
