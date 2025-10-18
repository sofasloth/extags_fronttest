<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../css/global.css';

	let { children } = $props();
	
	let isDarkMode = $state(true);
	
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

	// 하위 컴포넌트에서 사용할 수 있도록 window에 노출
	$effect(() => {
		if (typeof window !== 'undefined') {
			(window as any).__toggleTheme = toggleTheme;
			(window as any).__isDarkMode = isDarkMode;
		}
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<nav>
	<div class="nav-links">
		<a href="/">Home</a>
		<a href="/memo">Memo</a>
		<a href="/window">window</a>
		<a href="/file_interaction">File Interaction</a>
		<a href="/search_result">Search Result</a>
	</div>
	<button class="theme-toggle" onclick={toggleTheme} title="Toggle theme">
		{isDarkMode ? '☀️' : '🌙'}
	</button>
</nav>

{@render children?.()}

<style>
	nav {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.3rem 0.6rem;
		background: var(--bg-secondary);
		border-bottom: 1px solid var(--border-color);
		gap: 1rem;
		margin-bottom: 1rem;

		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 1000;
	}
	
	.nav-links {
		display: flex;
		gap: 1.5rem;
	}
	
	nav a {
		color: var(--text-primary);
		text-decoration: none;
		padding: 0.5rem 1rem;
		border-radius: 4px;
		transition: background-color 0.2s ease;
	}
	
	nav a:hover {
		background: var(--bg-tertiary);
	}
	
	.theme-toggle {
		padding: 0.5rem 1rem;
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
	
	.theme-toggle:hover {
		background: var(--btn-primary-hover);
		transform: scale(1.05);
	}
</style>
