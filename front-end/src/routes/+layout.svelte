<script module lang="ts">
	import { keepPreviousData, QueryClient } from '@tanstack/svelte-query';
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: 5 * 60 * 1000, // data fresh for 5 min
				gcTime: 30 * 60 * 1000, // cache kept for 30 min
				refetchOnWindowFocus: false, // revalidate on tab focus
				retry: 3,
				placeholderData: keepPreviousData // show old data while fetching new data
			}
		}
	});
</script>

<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import Navbar from '$lib/components/Navbar.svelte';
	import { Toaster } from '$lib/components/ui/sonner/index.js';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<QueryClientProvider client={queryClient}>
	<Navbar />
	<main class="mx-auto max-w-6xl px-4 py-6">
		{@render children()}
	</main>
	<Toaster richColors />
</QueryClientProvider>
