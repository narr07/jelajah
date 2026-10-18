<template>
	<div class="flex items-center justify-between px-3 py-1.5 bg-neutral-50 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 text-xs">
		<!-- Left command actions -->
		<div class="flex items-center gap-1">
			<!-- New dropdown -->
			<UDropdownMenu :items="newMenuItems" @update:open="(open: boolean) => open && loadTemplates()">
				<UButton
					variant="subtle"
					color="neutral"
					size="sm"
					icon="lucide:plus"
					trailing-icon="lucide:chevron-down"
				>
					New
				</UButton>
			</UDropdownMenu>

			<div class="h-4 w-px bg-neutral-300 dark:bg-neutral-700 mx-1" />

			<!-- Cut -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedCount > 0 ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="selectedCount === 0"
				title="Cut (Ctrl+X)"
				@click="handleCut"
			>
				<UIcon name="lucide:scissors" class="w-4 h-4" />
			</button>

			<!-- Copy -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedCount > 0 ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="selectedCount === 0"
				title="Copy (Ctrl+C)"
				@click="handleCopy"
			>
				<UIcon name="lucide:copy" class="w-4 h-4" />
			</button>

			<!-- Paste -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="clipboard ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="!clipboard"
				title="Paste (Ctrl+V)"
				@click="handlePaste"
			>
				<UIcon name="lucide:clipboard-paste" class="w-4 h-4" />
			</button>

			<!-- Rename -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedCount === 1 ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="selectedCount !== 1"
				title="Rename (F2)"
				@click="handleRename"
			>
				<UIcon name="lucide:pen-line" class="w-4 h-4" />
			</button>

			<!-- Delete -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedCount > 0 ? 'text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="selectedCount === 0"
				title="Delete (Del)"
				@click="handleDelete"
			>
				<UIcon name="lucide:trash-2" class="w-4 h-4" />
			</button>

			<div class="h-4 w-px bg-neutral-300 dark:bg-neutral-700 mx-1" />

			<!-- PowerToys: Quick Peek (Space) -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedCount >= 1 ? 'text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950/40' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="selectedCount === 0"
				title="Quick Peek (Space)"
				@click="togglePeek"
			>
				<UIcon name="lucide:eye" class="w-4 h-4" />
			</button>

			<!-- PowerToys: PowerRename (Ctrl+M) -->
			<button
				class="p-1.5 rounded-md transition-colors text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40"
				title="PowerRename Batch Renamer (Ctrl+M)"
				@click="openPowerRename"
			>
				<UIcon name="lucide:spell-check" class="w-4 h-4" />
			</button>

			<!-- PowerToys: Paste as File (Ctrl+Shift+V) -->
			<button
				class="p-1.5 rounded-md transition-colors text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800"
				title="Paste Clipboard as File (Ctrl+Shift+V)"
				@click="pasteAsFile"
			>
				<UIcon name="lucide:clipboard-paste" class="w-4 h-4" />
			</button>

			<!-- TreeSize: Disk Space Analyzer -->
			<button
				class="p-1.5 rounded-md transition-colors text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40"
				title="TreeSize Disk Space Analyzer (JAM Software style)"
				@click="openTreeSize()"
			>
				<UIcon name="lucide:pie-chart" class="w-4 h-4" />
			</button>

			<!-- Checksum / Hashes -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="selectedItem && !selectedItem.is_dir ? 'text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="!selectedItem || selectedItem.is_dir"
				title="File Checksum & Hash Inspector (MD5, SHA-256)"
				@click="openHashModal(selectedItem || undefined)"
			>
				<UIcon name="lucide:fingerprint" class="w-4 h-4" />
			</button>

			<!-- PowerToys: Image Resizer -->
			<button
				v-if="isImageSelected"
				class="p-1.5 rounded-md transition-colors text-violet-600 dark:text-violet-400 hover:bg-violet-50 dark:hover:bg-violet-950/40"
				title="PowerToys Image Resizer & Converter"
				@click="openImageResizer(selectedItem || undefined)"
			>
				<UIcon name="lucide:image-down" class="w-4 h-4" />
			</button>

			<!-- Dual Pane Toggle -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="isDualPane ? 'bg-primary-100 text-primary-700 dark:bg-primary-950/60 dark:text-primary-300' : 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800'"
				title="Toggle Dual-Pane Split View"
				@click="toggleDualPane"
			>
				<UIcon name="lucide:columns-2" class="w-4 h-4" />
			</button>

			<div class="h-4 w-px bg-neutral-300 dark:bg-neutral-700 mx-1" />

			<!-- Sort dropdown -->
			<UDropdownMenu :items="sortMenuItems">
				<UButton
					variant="ghost"
					color="neutral"
					size="sm"
					icon="lucide:arrow-up-down"
					trailing-icon="lucide:chevron-down"
				>
					Sort
				</UButton>
			</UDropdownMenu>

			<!-- View dropdown -->
			<UDropdownMenu :items="viewMenuItems">
				<UButton
					variant="ghost"
					color="neutral"
					size="sm"
					icon="lucide:layout-grid"
					trailing-icon="lucide:chevron-down"
				>
					View
				</UButton>
			</UDropdownMenu>
		</div>

		<!-- Right actions -->
		<div class="flex items-center gap-1.5">
			<!-- Fast tgrep Index button -->
			<UButton
				variant="soft"
				color="primary"
				size="xs"
				icon="lucide:zap"
				:loading="isIndexing"
				title="Index this folder with tgrep for instant searches"
				@click="handleIndexCurrentFolder"
			>
				Index with tgrep
			</UButton>

			<!-- Terminal / Command Prompt -->
			<button
				class="p-1.5 rounded-md text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
				title="Open Windows Terminal in this folder"
				@click="handleOpenTerminal"
			>
				<UIcon name="lucide:terminal" class="w-4 h-4" />
			</button>

			<!-- PowerToys Command Palette -->
			<button
				class="p-1.5 rounded-md text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
				title="Command Palette & Quick Launcher (Ctrl+P)"
				@click="toggleCommandPalette"
			>
				<UIcon name="lucide:terminal-square" class="w-4 h-4" />
			</button>

			<!-- Details / Preview Pane Toggle -->
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="showPreviewPane ? 'bg-primary-100 text-primary-700 dark:bg-primary-950/60 dark:text-primary-300' : 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-800'"
				title="Details & Preview pane"
				@click="showPreviewPane = !showPreviewPane"
			>
				<UIcon name="lucide:panel-right" class="w-4 h-4" />
			</button>
		</div>
	</div>
</template>

<script lang="ts" setup>
	const {
		currentPath,
		selectedPaths,
		selectedCount,
		viewMode,
		showHidden,
		showPreviewPane,
		sortBy,
		sortOrder,
		clipboard,
		refresh,
		items,
		togglePeek,
		openPowerRename,
		pasteAsFile,
		toggleCommandPalette,
		openTreeSize,
		openHashModal,
		openImageResizer,
		toggleDualPane,
		isDualPane,
		selectedItem,
		deletePaths
	} = useExplorer();

	const isImageSelected = computed(() => {
		if (!selectedItem.value || selectedItem.value.is_dir) return false;
		return ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'svg', 'avif'].includes(selectedItem.value.extension.toLowerCase());
	});

	const api = useExplorerApi();
	const toast = useToast();
	const isIndexing = ref(false);

	// PowerToys New+: files/folders in the templates folder show up under "New".
	const templates = ref<{ dir: string; items: { name: string; is_dir: boolean }[] }>({ dir: '', items: [] });
	const loadTemplates = async () => {
		try {
			templates.value = await api.newplusList();
		} catch {
			// keep the built-in entries usable
		}
	};
	const createFromTemplate = async (name: string) => {
		try {
			await api.newplusCreate(name, currentPath.value);
			refresh();
		} catch (e: any) {
			toast.add({ title: 'Error', description: e.message, color: 'error' });
		}
	};

	// New menu dropdown items
	const newMenuItems = computed(() => [
		[
			{
				label: 'Folder',
				icon: 'lucide:folder-plus',
				onSelect: async () => {
					try {
						await api.executeAction('createFolder', currentPath.value, { name: 'New folder' });
						refresh();
					} catch (e: any) {
						toast.add({ title: 'Error', description: e.message, color: 'error' });
					}
				}
			},
			{
				label: 'Text Document',
				icon: 'lucide:file-plus',
				onSelect: async () => {
					try {
						await api.executeAction('createFile', currentPath.value, { name: 'New Text Document.txt' });
						refresh();
					} catch (e: any) {
						toast.add({ title: 'Error', description: e.message, color: 'error' });
					}
				}
			}
		],
		templates.value.items.map(t => ({
			label: t.name,
			icon: t.is_dir ? 'lucide:folder-symlink' : 'lucide:file-symlink',
			onSelect: () => createFromTemplate(t.name)
		})),
		[
			{
				label: 'Open templates folder',
				icon: 'lucide:folder-cog',
				onSelect: () => templates.value.dir && api.executeAction('open', templates.value.dir)
			}
		]
	].filter(group => group.length));

	// Sort menu items
	const sortMenuItems = computed(() => [
		[
			{
				label: 'Name',
				icon: sortBy.value === 'name' ? 'lucide:check' : undefined,
				onSelect: () => sortBy.value = 'name'
			},
			{
				label: 'Date modified',
				icon: sortBy.value === 'modified' ? 'lucide:check' : undefined,
				onSelect: () => sortBy.value = 'modified'
			},
			{
				label: 'Type',
				icon: sortBy.value === 'type' ? 'lucide:check' : undefined,
				onSelect: () => sortBy.value = 'type'
			},
			{
				label: 'Size',
				icon: sortBy.value === 'size' ? 'lucide:check' : undefined,
				onSelect: () => sortBy.value = 'size'
			}
		],
		[
			{
				label: 'Ascending',
				icon: sortOrder.value === 'asc' ? 'lucide:check' : undefined,
				onSelect: () => sortOrder.value = 'asc'
			},
			{
				label: 'Descending',
				icon: sortOrder.value === 'desc' ? 'lucide:check' : undefined,
				onSelect: () => sortOrder.value = 'desc'
			}
		]
	]);

	// View menu items
	const viewMenuItems = computed(() => [
		[
			{
				label: 'Details',
				icon: viewMode.value === 'details' ? 'lucide:check' : 'lucide:table-properties',
				onSelect: () => viewMode.value = 'details'
			},
			{
				label: 'Large icons',
				icon: viewMode.value === 'large' ? 'lucide:check' : 'lucide:layout-grid',
				onSelect: () => viewMode.value = 'large'
			},
			{
				label: 'Grid tiles',
				icon: viewMode.value === 'grid' ? 'lucide:check' : 'lucide:grid-2x2',
				onSelect: () => viewMode.value = 'grid'
			}
		],
		[
			{
				label: 'Show hidden items',
				icon: showHidden.value ? 'lucide:check' : undefined,
				onSelect: () => {
					showHidden.value = !showHidden.value;
					refresh();
				}
			}
		]
	]);

	const handleCut = () => {
		clipboard.value = {
			paths: Array.from(selectedPaths.value),
			action: 'cut'
		};
		toast.add({ title: 'Cut', description: `${selectedPaths.value.size} item(s) ready to move` });
	};

	const handleCopy = () => {
		clipboard.value = {
			paths: Array.from(selectedPaths.value),
			action: 'copy'
		};
		toast.add({ title: 'Copied', description: `${selectedPaths.value.size} item(s) copied to clipboard` });
	};

	const handlePaste = async () => {
		if (!clipboard.value) return;
		try {
			for (const p of clipboard.value.paths) {
				const fileName = p.split(/[\\/]/).pop()!;
				const dest = `${currentPath.value}\\${fileName}`;
				if (clipboard.value.action === 'cut') {
					await api.executeAction('move', p, { destinationPath: dest });
				} else {
					await api.executeAction('copy', p, { destinationPath: dest });
				}
			}
			if (clipboard.value.action === 'cut') {
				clipboard.value = null;
			}
			refresh();
			toast.add({ title: 'Success', description: 'Paste operation completed' });
		} catch (e: any) {
			toast.add({ title: 'Error', description: e.message, color: 'error' });
		}
	};

	const handleRename = () => {
		// Emit or handle rename inline
		const targetPath = Array.from(selectedPaths.value)[0];
		const item = items.value.find(i => i.path === targetPath);
		if (!item) return;

		const newName = prompt('Enter new name:', item.name);
		if (newName && newName !== item.name) {
			const parent = item.path.replace(/[\\/][^\\/]+$/, '');
			const newFullPath = `${parent}\\${newName}`;
			api.executeAction('rename', item.path, { newPath: newFullPath })
				.then(() => refresh())
				.catch(err => toast.add({ title: 'Error', description: err.message, color: 'error' }));
		}
	};

	const handleDelete = async (e?: MouseEvent) => {
		try {
			const permanent = !!e?.shiftKey;
			if (await deletePaths([...selectedPaths.value], permanent)) {
				toast.add({ title: permanent ? 'Deleted' : 'Moved to Recycle Bin' });
			}
		} catch (e: any) {
			toast.add({ title: 'Error', description: e.message, color: 'error' });
		}
	};

	const handleOpenTerminal = () => {
		api.executeAction('openTerminal', currentPath.value);
	};

	const handleIndexCurrentFolder = async () => {
		isIndexing.value = true;
		try {
			const res = await api.buildTgrepIndex(currentPath.value);
			toast.add({
				title: res.success ? 'tgrep Index Built' : 'Indexing Notice',
				description: res.message || 'Folder successfully indexed for trigram search!',
				color: res.success ? 'success' : 'warning'
			});
		} catch (e: any) {
			toast.add({ title: 'Indexing failed', description: e.message, color: 'error' });
		} finally {
			isIndexing.value = false;
		}
	};
</script>
