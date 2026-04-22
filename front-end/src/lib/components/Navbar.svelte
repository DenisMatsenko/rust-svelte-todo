<script lang="ts">
	import { createMe } from '$lib/api/generated/auth/auth';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';

	const meQuery = createMe();
	const user = $derived(meQuery.data?.status === 200 ? meQuery.data.data : null);

	function getInitials(name: string) {
		return name
			.split(' ')
			.slice(0, 2)
			.map((n) => n[0])
			.join('')
			.toUpperCase();
	}
</script>

<header
	class="sticky top-0 z-50 w-full border-b border-border/60 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60"
>
	<div class="mx-auto flex h-14 max-w-6xl items-center gap-4 px-4">
		<!-- Logo / Brand -->
		<a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
			<div
				class="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-xs font-bold text-primary-foreground"
			>
				R
			</div>
			<span class="text-foreground">Relatoo</span>
		</a>

		<Separator orientation="vertical" class="h-5" />

		<!-- Nav links -->
		<nav class="flex items-center gap-1">
			<Button
				variant="ghost"
				size="sm"
				href="/"
				class="text-muted-foreground hover:text-foreground">Dashboard</Button
			>
			<Button
				variant="ghost"
				size="sm"
				href="/users"
				class="text-muted-foreground hover:text-foreground">Users</Button
			>
		</nav>

		<!-- Spacer -->
		<div class="flex-1"></div>

		<!-- Right side actions -->
		<div class="flex items-center gap-2">
			{#if meQuery.isLoading}
				<div class="h-8 w-8 animate-pulse rounded-full bg-muted"></div>
			{:else if user}
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						<Avatar.Root
							class="h-8 w-8 cursor-pointer ring-2 ring-transparent transition-all hover:ring-primary/40"
						>
							<Avatar.Fallback class="bg-primary/10 text-xs font-semibold text-primary">
								{getInitials(user.full_name)}
							</Avatar.Fallback>
						</Avatar.Root>
					</DropdownMenu.Trigger>
					<DropdownMenu.Content class="w-56" align="end">
						<DropdownMenu.Label class="font-normal">
							<div class="flex flex-col gap-0.5">
								<span class="text-sm font-medium">{user.full_name}</span>
								<span class="truncate text-xs text-muted-foreground">{user.email}</span>
							</div>
						</DropdownMenu.Label>
						<DropdownMenu.Separator />
						<DropdownMenu.Item class="text-destructive focus:text-destructive">
							<a href="/auth/signout" class="flex w-full items-center text-destructive">
								<svg
									xmlns="http://www.w3.org/2000/svg"
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
									class="mr-2"
								>
									<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline
										points="16 17 21 12 16 7"
									/><line x1="21" y1="12" x2="9" y2="12" />
								</svg>
								Sign out
							</a>
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			{:else}
				<Button variant="ghost" size="sm" href="/auth/signin">Sign in</Button>
				<Button size="sm" href="/auth/signup">Sign up</Button>
			{/if}
		</div>
	</div>
</header>
