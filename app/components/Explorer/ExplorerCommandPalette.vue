<template>
	<Teleport to="body">
		<Transition
			enter-active-class="transition duration-150 ease-out"
			enter-from-class="opacity-0 scale-95"
			enter-to-class="opacity-100 scale-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="opacity-100 scale-100"
			leave-to-class="opacity-0 scale-95"
		>
			<div
				v-if="isCommandPaletteOpen"
				class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] px-4 select-none"
				tabindex="0"
				ref="paletteRoot"
			>
				<!-- Independent Backdrop Layer -->
				<div class="absolute inset-0 bg-black/50 backdrop-blur-sm" @click="isCommandPaletteOpen = false" />

				<div class="relative z-10 w-full max-w-2xl bg-white dark:bg-neutral-900 rounded-2xl border border-neutral-200 dark:border-neutral-700 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100">
					<!-- Search Input Header -->
					<div class="flex items-center gap-3 px-4 py-3.5 border-b border-neutral-200/80 dark:border-neutral-800">
						<UIcon name="lucide:search" class="w-5 h-5 text-neutral-400 shrink-0" />
						<input
							ref="inputEl"
							v-model="query"
							type="text"
							placeholder="Type a folder, action, or drive (e.g. 'Desktop', 'Rename', 'C:')..."
							class="flex-1 bg-transparent text-sm focus:outline-none placeholder:text-neutral-400 text-neutral-900 dark:text-neutral-100 font-medium"
							@keydown="handleKeyDown"
						/>
						<kbd class="px-2 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-[11px] font-mono text-neutral-500">
							Esc
						</kbd>
					</div>

					<!-- Results List -->
					<div class="max-h-96 overflow-auto p-2 space-y-1">
						<div
							v-for="(item, idx) in filteredItems"
							:key="item.id"
							class="flex items-center justify-between px-3 py-2.5 rounded-xl cursor-pointer transition-colors"
							:class="idx === selectedIndex ? 'bg-primary-50 dark:bg-primary-950/60 text-primary-900 dark:text-primary-100 font-semibold' : 'hover:bg-neutral-100/70 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300'"
							@mouseenter="selectedIndex = idx"
							@click="executeItem(item)"
						>
							<div class="flex items-center gap-3 min-w-0">
								<div
									class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
									:class="idx === selectedIndex ? 'bg-primary-100 dark:bg-primary-900/80 text-primary-600 dark:text-primary-300' : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400'"
								>
									<UIcon :name="item.icon" class="w-4 h-4" />
								</div>
								<div class="min-w-0">
									<div class="text-xs truncate">{{ item.title }}</div>
									<div class="text-[11px] text-neutral-400 truncate font-normal font-mono">{{ item.subtitle }}</div>
								</div>
							</div>

							<div class="flex items-center gap-2 shrink-0">
								<span class="text-[10px] px-2 py-0.5 rounded-full bg-neutral-100 dark:bg-neutral-800 text-neutral-500 uppercase tracking-wider font-semibold">
									{{ item.category }}
								</span>
								<kbd v-if="idx === selectedIndex" class="px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900 font-mono text-[10px] text-primary-700 dark:text-primary-300">
									↵ Enter
								</kbd>
							</div>
						</div>

						<div v-if="filteredItems.length === 0" class="py-8 text-center text-xs text-neutral-400">
							No matching actions, drives, or folders found
						</div>
					</div>

					<!-- Bottom Hint -->
					<div class="px-4 py-2 border-t border-neutral-200/80 dark:border-neutral-800/80 bg-neutral-50/70 dark:bg-neutral-900/70 flex items-center justify-between text-[11px] text-neutral-500 font-medium">
						<span class="flex items-center gap-1.5">
							<kbd class="px-1 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono">↑</kbd>
							<kbd class="px-1 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono">↓</kbd>
							Navigate
						</span>
						<span class="flex items-center gap-1.5">
							<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono">Enter</kbd>
							Select
						</span>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script lang="ts" setup>
	interface PaletteItem {
		id: string;
		title: string;
		subtitle: string;
		icon: string;
		category: string;
		action: () => void;
	}

	const {
		isCommandPaletteOpen,
		drives,
		quickAccess,
		loadDirectory,
		currentPath,
		viewMode,
		showHidden,
		showPreviewPane,
		togglePeek,
		openPowerRename,
		pasteAsFile,
		triggerSearch
	} = useExplorer();

	const api = useExplorerApi();
	const toast = useToast();
	const query = ref('');
	const selectedIndex = ref(0);
	const paletteRoot = ref<HTMLElement | null>(null);
	const inputEl = ref<HTMLInputElement | null>(null);

	watch(isCommandPaletteOpen, (open) => {
		if (open) {
			query.value = '';
			selectedIndex.value = 0;
			nextTick(() => {
				inputEl.value?.focus();
			});
		}
	});

	watch(query, () => {
		selectedIndex.value = 0;
	});

	// Build available actions & locations
	const allItems = computed<PaletteItem[]>(() => {
		const items: PaletteItem[] = [];

		// PowerToys Tools & Actions
		items.push({
			id: 'tool-peek',
			title: 'Quick Peek (Space)',
			subtitle: 'Instantly preview selected file with syntax/image inspection',
			icon: 'lucide:eye',
			category: 'PowerToys',
			action: () => togglePeek()
		});

		items.push({
			id: 'tool-powerrename',
			title: 'PowerRename (Batch Rename)',
			subtitle: 'Regex, search & replace, and case conversion for multiple files',
			icon: 'lucide:spell-check',
			category: 'PowerToys',
			action: () => openPowerRename()
		});

		items.push({
			id: 'tool-paste-file',
			title: 'Paste as File (Ctrl+Shift+V)',
			subtitle: 'Save clipboard text or image directly as a new file in folder',
			icon: 'lucide:clipboard-paste',
			category: 'PowerToys',
			action: () => pasteAsFile()
		});

		items.push({
			id: 'action-new-folder',
			title: 'New Folder',
			subtitle: 'Create a new directory in current location',
			icon: 'lucide:folder-plus',
			category: 'Action',
			action: async () => {
				if (currentPath.value) {
					await api.executeAction('createFolder', currentPath.value);
					await loadDirectory(currentPath.value, false);
				}
			}
		});

		items.push({
			id: 'action-terminal',
			title: 'Open Terminal Here',
			subtitle: 'Launch PowerShell or Windows Terminal in current directory',
			icon: 'lucide:terminal',
			category: 'Action',
			action: () => {
				if (currentPath.value) {
					api.executeAction('openTerminal', currentPath.value);
				}
			}
		});

		items.push({
			id: 'action-index-tgrep',
			title: 'Re-index Folder with tgrep',
			subtitle: 'Build ultra-fast trigram index for current workspace',
			icon: 'lucide:zap',
			category: 'tgrep',
			action: () => {
				if (currentPath.value) {
					api.buildTgrepIndex(currentPath.value);
				}
			}
		});

		items.push({
			id: 'action-fast-index',
			title: 'Toggle Fast Index (NTFS MFT)',
			subtitle: 'Everything-style: rebuild the filename index in seconds via an elevated helper (one UAC prompt)',
			icon: 'lucide:rocket',
			category: 'Search',
			action: async () => {
				try {
					const on = await api.fastIndexStatus();
					if (!confirm(on ? 'Disable Fast Index? Windows will ask for admin.' : 'Enable Fast Index? Windows will ask for admin once.')) return;
					await api.setFastIndex(!on);
					toast.add({ title: on ? 'Fast Index disabled' : 'Fast Index enabled', description: on ? undefined : 'Rebuilding the filename index in the background', color: 'success' });
				} catch (e: any) {
					toast.add({ title: 'Fast Index', description: e.message, color: 'error' });
				}
			}
		});

		items.push({
			id: 'action-view-details',
			title: 'Switch to Details View',
			subtitle: 'Sortable table with columns for Name, Date, Type, Size',
			icon: 'lucide:list',
			category: 'View',
			action: () => { viewMode.value = 'details'; }
		});

		items.push({
			id: 'action-view-large',
			title: 'Switch to Large Icons View',
			subtitle: 'Grid thumbnail cards for media and documents',
			icon: 'lucide:grid',
			category: 'View',
			action: () => { viewMode.value = 'large'; }
		});

		items.push({
			id: 'action-toggle-hidden',
			title: showHidden.value ? 'Hide Hidden Items' : 'Show Hidden Items',
			subtitle: 'Toggle visibility of system and hidden files',
			icon: 'lucide:eye-off',
			category: 'View',
			action: () => { showHidden.value = !showHidden.value; }
		});

		items.push({
			id: 'action-toggle-preview',
			title: showPreviewPane.value ? 'Hide Preview Pane' : 'Show Preview Pane',
			subtitle: 'Toggle right-side inspector panel',
			icon: 'lucide:panel-right',
			category: 'View',
			action: () => { showPreviewPane.value = !showPreviewPane.value; }
		});

		// Drives
		for (const drive of drives.value) {
			items.push({
				id: `drive-${drive.letter}`,
				title: drive.label || `Drive (${drive.letter})`,
				subtitle: drive.path,
				icon: 'lucide:hard-drive',
				category: 'Drives',
				action: () => loadDirectory(drive.path)
			});
		}

		// Quick Access
		for (const qa of quickAccess.value) {
			items.push({
				id: `qa-${qa.name}`,
				title: qa.name,
				subtitle: qa.path,
				icon: 'lucide:folder-heart',
				category: 'Quick Access',
				action: () => loadDirectory(qa.path)
			});
		}

		return items;
	});

	// Filtered items based on query
	const filteredItems = computed(() => {
		if (!query.value.trim()) {
			return allItems.value;
		}
		const q = query.value.toLowerCase();
		return allItems.value.filter(item =>
			item.title.toLowerCase().includes(q) ||
			item.subtitle.toLowerCase().includes(q) ||
			item.category.toLowerCase().includes(q)
		);
	});

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			isCommandPaletteOpen.value = false;
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (filteredItems.value.length > 0) {
				selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length;
			}
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (filteredItems.value.length > 0) {
				selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length;
			}
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const selected = filteredItems.value[selectedIndex.value];
			if (selected) {
				executeItem(selected);
			}
		}
	};

	const executeItem = (item: PaletteItem) => {
		isCommandPaletteOpen.value = false;
		item.action();
	};
</script>
