<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient, createQueries } from '@tanstack/svelte-query';
	import {
		createListTenants,
		getGetTenantFansCountQueryOptions
	} from '$lib/api/generated/tenants/tenants';
	import { createListBillingEntries } from '$lib/api/generated/billing/billing';
	import {
		createUpsertTenantNote,
		createDeleteTenantNote,
		getGetTenantNoteQueryKey,
		getGetTenantNoteQueryOptions
	} from '$lib/api/generated/tenant-notes/tenant-notes';
	import type { BillingEntry, TenantResponse } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { BillingCondition } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';

	const queryClient = useQueryClient();

	// ── Tenants ────────────────────────────────────────────────────────────────
	const tenantsQuery = createListTenants();
	const allTenants = $derived(tenantsQuery.data?.status === 200 ? tenantsQuery.data.data : []);

	let search = $state('');

	function normalize(s: string) {
		return s
			.normalize('NFD')
			.replace(/\p{Diacritic}/gu, '')
			.toLowerCase();
	}

	const tenants = $derived(
		search.trim()
			? allTenants.filter((t) => normalize(t.name).includes(normalize(search.trim())))
			: allTenants
	);

	function planVariant(plan: string | null | undefined): 'default' | 'secondary' | 'outline' {
		if (plan === 'PRO') return 'default';
		if (plan === 'BASIC') return 'secondary';
		return 'outline';
	}

	type Segment = { text: string; match: boolean };

	function highlight(name: string, query: string): Segment[] {
		const q = normalize(query.trim());
		if (!q) return [{ text: name, match: false }];
		const normName = normalize(name);
		const segments: Segment[] = [];
		let i = 0;
		while (i < name.length) {
			const idx = normName.indexOf(q, i);
			if (idx === -1) {
				segments.push({ text: name.slice(i), match: false });
				break;
			}
			if (idx > i) segments.push({ text: name.slice(i, idx), match: false });
			segments.push({ text: name.slice(idx, idx + q.length), match: true });
			i = idx + q.length;
		}
		return segments;
	}

	// ── Parallel note fetch for every tenant ───────────────────────────────────
	const noteQueries = createQueries(() => ({
		queries: allTenants.map((t) => getGetTenantNoteQueryOptions(t.id))
	}));

	// ── Billing entries (used for scope calculation) ───────────────────────────
	const billingQuery = createListBillingEntries();
	const billingEntries = $derived(billingQuery.data?.status === 200 ? billingQuery.data.data : []);

	/// Returns the billing entry for the tier the given fan count falls into.
	/// Finds the smallest "less_than" threshold still greater than `fans`.
	/// Falls back to the entry with the largest threshold if fans exceed everything.
	function getScopeBillingEntry(fans: number, entries: BillingEntry[]): BillingEntry | null {
		if (entries.length === 0) return null;
		const lessThan = entries
			.filter((e) => e.condition === BillingCondition.less_than)
			.sort((a, b) => a.fans_count - b.fans_count);
		const match = lessThan.find((e) => e.fans_count > fans);
		if (match) return match;
		return entries.reduce((max, e) => (e.fans_count > max.fans_count ? e : max), entries[0]);
	}

	function fillColor(value: number, max: number): string {
		const ratio = Math.min(value / max, 1);
		const hue = 120 * Math.sin(ratio * Math.PI);
		return `hsl(${hue}, 72%, 42%)`;
	}

	function getScopeThreshold(fans: number, entries: BillingEntry[]): number | null {
		return getScopeBillingEntry(fans, entries)?.fans_count ?? null;
	}

	/// Maps a tenant plan string to the corresponding price from a billing entry.
	function getExpectedPrice(
		fans: number | null,
		plan: string | null | undefined,
		entries: BillingEntry[]
	): number | null {
		if (fans === null || !plan) return null;
		const entry = getScopeBillingEntry(fans, entries);
		if (!entry) return null;
		const p = plan.toUpperCase();
		if (p === 'BASIC') return entry.basic_plan_price ?? null;
		if (p === 'STANDARD') return entry.standard_plan_price ?? null;
		if (p === 'PRO' || p === 'PREMIUM') return entry.premium_plan_price ?? null;
		return null;
	}

	// ── Parallel fans count fetch for every tenant ─────────────────────────────
	const fansQueries = createQueries(() => ({
		queries: allTenants.map((t) => getGetTenantFansCountQueryOptions(t.id))
	}));

	function getFansCount(tenantIndex: number): number | null {
		const q = fansQueries[tenantIndex];
		return q?.data?.status === 200 ? q.data.data.fans_count : null;
	}

	function getNoteText(tenantIndex: number): string | null {
		const q = noteQueries[tenantIndex];
		return q?.data?.status === 200 ? (q.data.data.note ?? null) : null;
	}

	function getNotePrice(tenantIndex: number): number | null {
		const q = noteQueries[tenantIndex];
		return q?.data?.status === 200 ? (q.data.data.price ?? null) : null;
	}

	// ── Edit note ──────────────────────────────────────────────────────────────
	let editOpen = $state(false);
	let editingTenant = $state<TenantResponse | null>(null);
	let editNoteText = $state('');
	let editNotePrice = $state<number | null>(null);

	function openEdit(tenant: TenantResponse, tenantIndex: number) {
		editingTenant = tenant;
		editNoteText = getNoteText(tenantIndex) ?? '';
		editNotePrice = getNotePrice(tenantIndex);
		editOpen = true;
	}

	const upsertMutation = createUpsertTenantNote(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({
					queryKey: getGetTenantNoteQueryKey(editingTenant!.id)
				});
				toast.success('Note saved');
				editOpen = false;
				editingTenant = null;
			},
			onError: () => toast.error('Failed to save note')
		}
	}));

	function submitEdit(e: SubmitEvent) {
		e.preventDefault();
		if (!editingTenant) return;
		upsertMutation.mutate({
			mongoId: editingTenant.id,
			data: { note: editNoteText.trim() || null, price: editNotePrice ?? null }
		});
	}

	// ── Delete note ────────────────────────────────────────────────────────────
	let deleteOpen = $state(false);
	let deletingTenant = $state<TenantResponse | null>(null);

	function openDelete(tenant: TenantResponse) {
		deletingTenant = tenant;
		deleteOpen = true;
	}

	const deleteMutation = createDeleteTenantNote(() => ({
		mutation: {
			onSuccess: () => {
				if (deletingTenant) {
					queryClient.invalidateQueries({
						queryKey: getGetTenantNoteQueryKey(deletingTenant.id)
					});
				}
				toast.success('Note deleted');
				deleteOpen = false;
				deletingTenant = null;
			},
			onError: () => toast.error('Failed to delete note')
		}
	}));
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Overview</h1>
			<p class="text-sm text-muted-foreground">Admin notes attached to individual tenants.</p>
		</div>
		<Input bind:value={search} placeholder="Search by name…" class="w-64" />
	</div>

	{#if tenantsQuery.isPending}
		<div class="text-sm text-muted-foreground">Loading…</div>
	{:else if tenantsQuery.isError}
		<div class="text-sm text-destructive">Failed to load tenants.</div>
	{:else}
		<div class="rounded-lg border">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head class="w-12 text-center">#</Table.Head>
						<Table.Head class="w-12"></Table.Head>
						<Table.Head class="w-56">Name</Table.Head>
						<Table.Head class="w-24">Plan</Table.Head>
						<Table.Head class="w-44 text-right">Fans / Scope</Table.Head>
						<Table.Head class="w-32 min-w-32">Fill</Table.Head>
						<Table.Head class="w-32 text-right">Expected price</Table.Head>
						<Table.Head class="w-32 text-right">Custom price</Table.Head>
						<Table.Head class="w-36 text-right">Diff</Table.Head>
						<Table.Head>Note</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each tenants as tenant, i (tenant.id)}
						{@const ai = allTenants.findIndex((t) => t.id === tenant.id)}
						{@const note = getNoteText(ai)}
						{@const notePrice = getNotePrice(ai)}
						{@const isPending = noteQueries[ai]?.isPending ?? true}
						<ContextMenu.Root>
							<ContextMenuPrimitive.Trigger>
								{#snippet child({ props })}
									<Table.Row {...props} class="cursor-context-menu select-none">
										<Table.Cell class="text-center text-xs text-muted-foreground">
											{i + 1}
										</Table.Cell>
										<Table.Cell>
											{#if tenant.club_logo}
												<img
													src={tenant.club_logo}
													alt={tenant.name}
													class="h-8 w-8 rounded-full object-contain"
												/>
											{:else}
												<div
													class="flex h-8 w-8 items-center justify-center rounded-full bg-muted text-xs font-semibold text-muted-foreground"
												>
													{tenant.name.slice(0, 2).toUpperCase()}
												</div>
											{/if}
										</Table.Cell>
										<Table.Cell class="font-medium">
											{#each highlight(tenant.name, search) as seg}
												{#if seg.match}
													<mark
														class="rounded bg-yellow-300/60 px-0.5 text-inherit dark:bg-yellow-500/40"
														>{seg.text}</mark
													>
												{:else}
													{seg.text}
												{/if}
											{/each}
										</Table.Cell>
										<Table.Cell>
											{#if tenant.plan}
												<Badge variant={planVariant(tenant.plan)}>{tenant.plan}</Badge>
											{:else}
												<span class="text-muted-foreground">—</span>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-right font-mono text-muted-foreground tabular-nums">
											{#if fansQueries[ai]?.isPending ?? true}
												<span class="text-xs text-muted-foreground/50">…</span>
											{:else}
												{@const fans = getFansCount(ai)}
												{@const threshold =
													fans !== null ? getScopeThreshold(fans, billingEntries) : null}
												{#if fans !== null && threshold !== null}
													{fans.toLocaleString()} / {threshold.toLocaleString()}
												{:else}
													<span class="text-muted-foreground">—</span>
												{/if}
											{/if}
										</Table.Cell>
										<Table.Cell class="w-32 min-w-32 pr-4">
											{#if fansQueries[ai]?.isPending ?? true}
												<div class="h-3 w-full animate-pulse rounded-full bg-muted"></div>
											{:else}
												{@const fans = getFansCount(ai)}
												{@const threshold =
													fans !== null ? getScopeThreshold(fans, billingEntries) : null}
												{#if fans !== null && threshold !== null}
													<div class="relative h-2 w-full overflow-hidden rounded-full bg-muted">
														<div
															class="h-full rounded-full transition-all"
															style="width: {Math.min(
																(fans / threshold) * 100,
																100
															)}%; background-color: {fillColor(fans, threshold)};"
														></div>
													</div>
												{:else}
													<span class="text-xs text-muted-foreground">—</span>
												{/if}
											{/if}
										</Table.Cell>
										<Table.Cell class="text-right font-mono tabular-nums">
											{#if fansQueries[ai]?.isPending ?? true}
												<span class="text-xs text-muted-foreground/50">…</span>
											{:else}
												{@const price = getExpectedPrice(
													getFansCount(ai),
													tenant.plan,
													billingEntries
												)}
												{#if price !== null}
													{price.toLocaleString()}
												{:else}
													<span class="text-muted-foreground">—</span>
												{/if}
											{/if}
										</Table.Cell>
										<Table.Cell class="text-right font-mono tabular-nums">
											{#if isPending}
												<span class="text-xs text-muted-foreground/50">…</span>
											{:else if notePrice !== null}
												{notePrice.toLocaleString()}
											{:else}
												<span class="text-muted-foreground">—</span>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-right font-mono tabular-nums">
											{#if !isPending && notePrice !== null && (() => {
													const ep = getExpectedPrice(getFansCount(ai), tenant.plan, billingEntries);
													return ep !== null && ep !== 0 && notePrice !== ep ? { diff: notePrice - ep, pct: Math.round(((notePrice - ep) / ep) * 100) } : null;
												})() !== null}
												{@const ep = getExpectedPrice(
													getFansCount(ai),
													tenant.plan,
													billingEntries
												)!}
												{@const diff = notePrice - ep}
												{@const pct = Math.round((diff / ep) * 100)}
												{#if diff > 0}
													<span class="text-purple-500">+{diff.toLocaleString()} ({pct}%)</span>
												{:else}
													<span class="text-red-500">{diff.toLocaleString()} ({pct}%)</span>
												{/if}
											{/if}
										</Table.Cell>
										<Table.Cell>
											{#if isPending}
												<span class="text-xs text-muted-foreground/50">Loading…</span>
											{:else if note}
												<span class="line-clamp-2 text-sm">{note}</span>
											{:else}
												<span class="text-xs text-muted-foreground/40">—</span>
											{/if}
										</Table.Cell>
									</Table.Row>
								{/snippet}
							</ContextMenuPrimitive.Trigger>
							<ContextMenu.Content>
								<ContextMenu.Label class="text-xs text-muted-foreground">
									{tenant.name}
								</ContextMenu.Label>
								<ContextMenu.Separator />
								<ContextMenu.Item onclick={() => openEdit(tenant, ai)}>Edit Note</ContextMenu.Item>
								<ContextMenu.Separator />
								<ContextMenu.Item
									class="text-destructive focus:text-destructive"
									onclick={() => openDelete(tenant)}
								>
									Delete Note
								</ContextMenu.Item>
							</ContextMenu.Content>
						</ContextMenu.Root>
					{:else}
						<Table.Row>
							<Table.Cell class="py-8 text-center text-muted-foreground" colspan={10}>
								No tenants found.
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{/if}
</div>

<!-- Edit Note Dialog -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content class="sm:max-w-[480px]">
		<Dialog.Header>
			<Dialog.Title>Edit Note</Dialog.Title>
			<Dialog.Description>Note for <strong>{editingTenant?.name}</strong>.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitEdit}>
			<input
				type="number"
				bind:value={editNotePrice}
				placeholder="Custom price (integer)…"
				class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
			/>
			<textarea
				bind:value={editNoteText}
				placeholder="Write a note…"
				rows={6}
				class="w-full resize-y rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
			></textarea>
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={upsertMutation.isPending}>
					{upsertMutation.isPending ? 'Saving…' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirm Dialog -->
<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete Note</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete the note for
				<strong>{deletingTenant?.name}</strong>? This cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Cancel</Button>
			<Button
				variant="destructive"
				disabled={deleteMutation.isPending}
				onclick={() => {
					if (deletingTenant) deleteMutation.mutate({ mongoId: deletingTenant.id });
				}}
			>
				{deleteMutation.isPending ? 'Deleting…' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
