<script lang="ts">
	import { createMe } from '$lib/api/generated/users/users';

	const meQuery = createMe();

	const user = $derived(meQuery.data?.status === 200 ? meQuery.data.data : null);
</script>

<h1>Hello fwefwefw hweofkweof</h1>
{#if meQuery.isPending}
	<p>Loading...</p>
{:else if user}
	<h1>Hello {user.full_name}</h1>
	<p>id: {user.id}</p>
	<p>email: {user.email}</p>
{:else}
	<p>Failed to load user. <a href="/auth/signin">Sign in again</a></p>
{/if}
