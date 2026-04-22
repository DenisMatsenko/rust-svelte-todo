<script lang="ts">
	import { toast } from 'svelte-sonner';
	import { useQueryClient } from '@tanstack/svelte-query';
	import {
		createListUsers,
		createCreateUser,
		createUpdateUser,
		createDeleteUser,
		getListUsersQueryKey
	} from '$lib/api/generated/users/users';
	import { UserRole } from '$lib/api/generated/rustSvelteTodo.schemas';
	import type { User } from '$lib/api/generated/rustSvelteTodo.schemas';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import * as ContextMenu from '$lib/components/ui/context-menu/index.js';
	import { ContextMenu as ContextMenuPrimitive } from 'bits-ui';

	const queryClient = useQueryClient();
	const usersQuery = createListUsers();
	const users = $derived(usersQuery.data?.status === 200 ? usersQuery.data.data : []);

	const ROLE_LABELS: Record<string, string> = {
		[UserRole.admin]: 'Admin',
		[UserRole.editor]: 'Editor',
		[UserRole.viewer]: 'Viewer'
	};

	function roleBadgeVariant(role: string): 'default' | 'secondary' | 'outline' {
		if (role === UserRole.admin) return 'default';
		if (role === UserRole.editor) return 'secondary';
		return 'outline';
	}

	// ── Add user ────────────────────────────────────────────────────────────────
	let addOpen = $state(false);
	let addForm = $state({ full_name: '', email: '', password: '', role: UserRole.viewer as string });
	let addError = $state('');

	const addMutation = createCreateUser(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
				toast.success('User created');
				addOpen = false;
				addForm = { full_name: '', email: '', password: '', role: UserRole.viewer };
				addError = '';
			},
			onError: (err) => {
				addError = err.error ?? 'Failed to create user';
				toast.error(err.error ?? 'Failed to create user');
			}
		}
	}));

	function submitAdd(e: SubmitEvent) {
		e.preventDefault();
		addMutation.mutate({
			data: {
				full_name: addForm.full_name,
				email: addForm.email,
				password: addForm.password,
				role: addForm.role as (typeof UserRole)[keyof typeof UserRole]
			}
		});
	}

	// ── Edit user ────────────────────────────────────────────────────────────────
	let editOpen = $state(false);
	let editingUser = $state<User | null>(null);
	let editForm = $state({ full_name: '', role: UserRole.viewer as string });
	let editError = $state('');

	function openEdit(user: User) {
		editingUser = user;
		editForm = { full_name: user.full_name, role: user.role };
		editError = '';
		editOpen = true;
	}

	const editMutation = createUpdateUser(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
				toast.success('User updated');
				editOpen = false;
				editingUser = null;
				editError = '';
			},
			onError: (err) => {
				editError = err.error ?? 'Failed to update user';
				toast.error(err.error ?? 'Failed to update user');
			}
		}
	}));

	function submitEdit(e: SubmitEvent) {
		e.preventDefault();
		if (!editingUser) return;
		editMutation.mutate({
			id: editingUser.id,
			data: {
				full_name: editForm.full_name,
				role: editForm.role as (typeof UserRole)[keyof typeof UserRole]
			}
		});
	}

	// ── Delete user ───────────────────────────────────────────────────────────────
	let deleteOpen = $state(false);
	let deletingUser = $state<User | null>(null);

	function openDelete(user: User) {
		deletingUser = user;
		deleteOpen = true;
	}

	const deleteMutation = createDeleteUser(() => ({
		mutation: {
			onSuccess: () => {
				queryClient.invalidateQueries({ queryKey: getListUsersQueryKey() });
				toast.success('User deleted');
				deleteOpen = false;
				deletingUser = null;
			},
			onError: () => {
				toast.error('Failed to delete user');
			}
		}
	}));
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Users</h1>
			<p class="text-sm text-muted-foreground">Manage user accounts and roles.</p>
		</div>
		<Button onclick={() => (addOpen = true)}>+ Add User</Button>
	</div>

	<!-- Table -->
	{#if usersQuery.isPending}
		<div class="text-sm text-muted-foreground">Loading...</div>
	{:else if usersQuery.isError}
		<div class="text-sm text-destructive">Failed to load users.</div>
	{:else}
		<div class="rounded-lg border">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Name</Table.Head>
						<Table.Head>Email</Table.Head>
						<Table.Head>Role</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each users as user (user.id)}
						<ContextMenu.Root>
							<ContextMenuPrimitive.Trigger>
								{#snippet child({ props })}
									<Table.Row {...props} class="cursor-context-menu select-none">
										<Table.Cell class="font-medium">{user.full_name}</Table.Cell>
										<Table.Cell class="text-muted-foreground">{user.email}</Table.Cell>
										<Table.Cell>
											<Badge variant={roleBadgeVariant(user.role)}>
												{ROLE_LABELS[user.role] ?? user.role}
											</Badge>
										</Table.Cell>
									</Table.Row>
								{/snippet}
							</ContextMenuPrimitive.Trigger>
							<ContextMenu.Content>
								<ContextMenu.Label class="text-xs text-muted-foreground">
									{user.full_name}
								</ContextMenu.Label>
								<ContextMenu.Separator />
								<ContextMenu.Item onclick={() => openEdit(user)}>Edit</ContextMenu.Item>
								<ContextMenu.Separator />
								<ContextMenu.Item
									class="text-destructive focus:text-destructive"
									onclick={() => openDelete(user)}
								>
									Delete
								</ContextMenu.Item>
							</ContextMenu.Content>
						</ContextMenu.Root>
					{:else}
						<Table.Row>
							<Table.Cell class="py-8 text-center text-muted-foreground" colspan={3}>
								No users found.
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{/if}
</div>

<!-- Add User Dialog -->
<Dialog.Root bind:open={addOpen}>
	<Dialog.Content class="sm:max-w-[440px]">
		<Dialog.Header>
			<Dialog.Title>Add User</Dialog.Title>
			<Dialog.Description>Create a new user account.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitAdd}>
			<div class="space-y-1.5">
				<Label for="add-name">Full Name</Label>
				<Input id="add-name" bind:value={addForm.full_name} placeholder="Jane Doe" required />
			</div>
			<div class="space-y-1.5">
				<Label for="add-email">Email</Label>
				<Input
					id="add-email"
					type="email"
					bind:value={addForm.email}
					placeholder="jane@example.com"
					required
				/>
			</div>
			<div class="space-y-1.5">
				<Label for="add-password">Password</Label>
				<Input
					id="add-password"
					type="password"
					bind:value={addForm.password}
					placeholder="••••••••"
					required
				/>
			</div>
			<div class="space-y-1.5">
				<Label>Role</Label>
				<Select.Root type="single" bind:value={addForm.role}>
					<Select.Trigger class="w-full">
						{ROLE_LABELS[addForm.role] ?? 'Select role'}
					</Select.Trigger>
					<Select.Content>
						{#each Object.values(UserRole) as role (role)}
							<Select.Item value={role}>{ROLE_LABELS[role]}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			{#if addError}
				<p class="text-sm text-destructive">{addError}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (addOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={addMutation.isPending}>
					{addMutation.isPending ? 'Creating...' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit User Dialog -->
<Dialog.Root bind:open={editOpen}>
	<Dialog.Content class="sm:max-w-[440px]">
		<Dialog.Header>
			<Dialog.Title>Edit User</Dialog.Title>
			<Dialog.Description>
				Update name and role for {editingUser?.full_name ?? ''}.
			</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-2" onsubmit={submitEdit}>
			<div class="space-y-1.5">
				<Label for="edit-name">Full Name</Label>
				<Input id="edit-name" bind:value={editForm.full_name} required />
			</div>
			<div class="space-y-1.5">
				<Label>Role</Label>
				<Select.Root type="single" bind:value={editForm.role}>
					<Select.Trigger class="w-full">
						{ROLE_LABELS[editForm.role] ?? 'Select role'}
					</Select.Trigger>
					<Select.Content>
						{#each Object.values(UserRole) as role (role)}
							<Select.Item value={role}>{ROLE_LABELS[role]}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			{#if editError}
				<p class="text-sm text-destructive">{editError}</p>
			{/if}
			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={editMutation.isPending}>
					{editMutation.isPending ? 'Saving...' : 'Save'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirm Dialog -->
<Dialog.Root bind:open={deleteOpen}>
	<Dialog.Content class="sm:max-w-[400px]">
		<Dialog.Header>
			<Dialog.Title>Delete User</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to delete <strong>{deletingUser?.full_name}</strong>? This cannot be
				undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (deleteOpen = false)}>Cancel</Button>
			<Button
				variant="destructive"
				disabled={deleteMutation.isPending}
				onclick={() => {
					if (deletingUser) deleteMutation.mutate({ id: deletingUser.id });
				}}
			>
				{deleteMutation.isPending ? 'Deleting...' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
