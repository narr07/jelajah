<template>
	<div class="h-screen w-screen flex flex-col overflow-hidden bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 antialiased font-sans">
		<!-- Windows 11 Tabbed Titlebar -->
		<ExplorerTabs />

		<!-- Navigation & Breadcrumb Address Bar + tgrep search -->
		<ExplorerNavAddressBar />

		<!-- Command Bar (New, Cut, Copy, Rename, Delete, Sort, View, tgrep Index, PowerToys tools, TreeSize) -->
		<ExplorerCommandBar />

		<!-- Main Workspace Area -->
		<div class="flex-1 flex min-h-0 overflow-hidden">
			<!-- Left Navigation Pane (Quick Access + Drives) -->
			<ExplorerSidebar />

			<!-- Central Files Workspace (supports Single or Dual-Pane Split View) -->
			<div class="flex-1 flex min-w-0 min-h-0 overflow-hidden">
				<!-- Primary Pane -->
				<ExplorerFileList
					class="min-w-0"
					:class="isDualPane ? 'w-1/2 border-r border-neutral-200 dark:border-neutral-800' : 'flex-1'"
				/>

				<!-- Secondary Pane (Dual-Pane Mode) -->
				<div
					v-if="isDualPane"
					class="w-1/2 flex flex-col min-w-0 bg-white dark:bg-neutral-900 overflow-hidden select-none"
				>
					<!-- Secondary Pane Header -->
					<div class="px-3 py-1.5 bg-neutral-100/90 dark:bg-neutral-800/90 border-b border-neutral-200 dark:border-neutral-700 flex items-center justify-between text-xs">
						<div class="flex items-center gap-1.5 truncate max-w-sm">
							<UIcon name="lucide:columns-2" class="w-3.5 h-3.5 text-primary-500 shrink-0" />
							<span class="font-semibold text-neutral-700 dark:text-neutral-300">Dual Pane:</span>
							<span class="truncate font-mono text-[11px] text-neutral-500">{{ dualPanePath }}</span>
						</div>
						<div class="flex items-center gap-1">
							<button
								class="p-1 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-500"
								title="Close Dual Pane"
								@click="toggleDualPane"
							>
								<UIcon name="lucide:x" class="w-3.5 h-3.5" />
							</button>
						</div>
					</div>

					<!-- Secondary Pane File List -->
					<div class="flex-1 overflow-y-auto divide-y divide-neutral-100 dark:divide-neutral-800/50 text-xs">
						<div
							v-for="item in dualPaneItems"
							:key="item.path"
							class="flex items-center h-8 px-3 cursor-pointer hover:bg-neutral-50 dark:hover:bg-neutral-800/60 transition-colors"
							@dblclick="handleDualPaneDblClick(item)"
						>
							<UIcon
								:name="item.is_dir ? 'lucide:folder' : 'lucide:file'"
								class="w-4 h-4 mr-2.5 shrink-0"
								:class="item.is_dir ? 'text-amber-500' : 'text-neutral-400'"
							/>
							<span class="truncate flex-1 font-medium">{{ item.name }}</span>
							<span class="text-neutral-400 font-mono text-[11px] ml-2 shrink-0">
								{{ item.is_dir ? 'Folder' : formatBytes(item.size) }}
							</span>
						</div>

						<div v-if="dualPaneItems.length === 0" class="py-12 text-center text-xs text-neutral-400">
							This folder is empty.
						</div>
					</div>
				</div>
			</div>

			<!-- Collapsible Right Details & Preview Pane -->
			<ExplorerPreviewPane />
		</div>

		<!-- Windows 11 Status Bar -->
		<ExplorerStatusBar />

		<!-- PowerToys & TreeSize Modals (ClientOnly because they Teleport to body) -->
		<ClientOnly>
			<ExplorerPeekModal />
			<ExplorerPowerRenameModal />
			<ExplorerCommandPalette />
			<ExplorerTreeSizeModal />
			<ExplorerHashModal />
			<ExplorerImageResizerModal />
		</ClientOnly>
	</div>
</template>

<script lang="ts" setup>
	import type { FileItem } from '~/composables/useExplorerApi';

	definePageMeta({
		layout: 'fullscreen'
	});

	const {
		initialize,
		togglePeek,
		openPowerRename,
		pasteAsFile,
		toggleCommandPalette,
		openTreeSize,
		isPeekOpen,
		isPowerRenameOpen,
		isCommandPaletteOpen,
		isTreeSizeOpen,
		isHashModalOpen,
		isImageResizerOpen,
		isDualPane,
		dualPanePath,
		dualPaneItems,
		loadDualPaneDirectory,
		toggleDualPane,
		selectedPaths,
		selectedItem,
		deletePaths,
		refresh,
		goBack,
		goForward,
		goUp,
		addTab,
		closeTab,
		activeTabId,
		currentPath
	} = useExplorer();

	const api = useExplorerApi();
	const toast = useToast();
	const report = (p: Promise<unknown>) => p.catch((err: Error) => toast.add({ title: 'Error', description: err.message, color: 'error' }));

	// F2, same flow as the command bar's Rename
	const renameSelected = () => {
		const item = selectedItem.value;
		if (!item) return;
		const newName = prompt('Rename', item.name);
		if (!newName || newName === item.name) return;
		const parent = item.path.replace(/[\\/][^\\/]+$/, '');
		report(api.executeAction('rename', item.path, { newPath: `${parent}\\${newName}` }).then(refresh));
	};

	const handleDualPaneDblClick = (item: FileItem) => {
		if (item.is_dir) {
			loadDualPaneDirectory(item.path);
		}
	};

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
	}

	const handleGlobalKeyDown = (e: KeyboardEvent) => {
		const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
		const isInput = tag === 'input' || tag === 'textarea';

		// Command Palette shortcut: Ctrl+P or Ctrl+K (works everywhere)
		if ((e.key === 'p' || e.key === 'k') && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			toggleCommandPalette();
			return;
		}

		if (isInput) return;

		// PowerToys Quick Peek shortcut: Spacebar
		if (e.code === 'Space' && !isPeekOpen.value && !isPowerRenameOpen.value && !isCommandPaletteOpen.value && !isTreeSizeOpen.value && !isHashModalOpen.value && !isImageResizerOpen.value) {
			e.preventDefault();
			togglePeek();
		}
		// PowerToys PowerRename shortcut: Ctrl+M
		else if (e.key === 'm' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			openPowerRename();
		}
		// PowerToys Paste as File: Ctrl+Shift+V
		else if (e.key.toLowerCase() === 'v' && (e.ctrlKey || e.metaKey) && e.shiftKey) {
			e.preventDefault();
			pasteAsFile();
		}
		// TreeSize: Alt+T (Ctrl+T is "new tab", as in explorer.exe)
		else if ((e.key === 't' || e.key === 'T') && e.altKey) {
			e.preventDefault();
			openTreeSize();
		}
		// ---- explorer.exe shortcuts ----
		else if (e.key === 'Delete') {
			e.preventDefault();
			report(deletePaths([...selectedPaths.value], e.shiftKey));
		}
		else if (e.key === 'F10' && e.shiftKey && selectedItem.value) {
			e.preventDefault();
			report(api.shellContextMenu(selectedItem.value.path).then(refresh));
		}
		else if (e.key === 'Enter' && e.altKey && selectedItem.value) {
			e.preventDefault();
			report(api.showProperties(selectedItem.value.path));
		}
		else if (e.key === 'F2') {
			e.preventDefault();
			renameSelected();
		}
		else if (e.key === 'F5' || (e.key === 'r' && e.ctrlKey)) {
			e.preventDefault();
			refresh();
		}
		else if ((e.key === 'ArrowLeft' && e.altKey) || e.key === 'Backspace') {
			e.preventDefault();
			goBack();
		}
		else if (e.key === 'ArrowRight' && e.altKey) {
			e.preventDefault();
			goForward();
		}
		else if (e.key === 'ArrowUp' && e.altKey) {
			e.preventDefault();
			goUp();
		}
		else if (e.key === 't' && e.ctrlKey) {
			e.preventDefault();
			addTab(currentPath.value);
		}
		else if (e.key === 'w' && e.ctrlKey) {
			e.preventDefault();
			closeTab(activeTabId.value);
		}
		else if (e.key === 'N' && e.ctrlKey && e.shiftKey) {
			e.preventDefault();
			report(api.executeAction('createFolder', currentPath.value).then(refresh));
		}
		// Dual Pane toggle: F6
		else if (e.key === 'F6') {
			e.preventDefault();
			toggleDualPane();
		}
	};

	onMounted(() => {
		initialize();
		window.addEventListener('keydown', handleGlobalKeyDown);
	});

	onUnmounted(() => {
		window.removeEventListener('keydown', handleGlobalKeyDown);
	});
</script>
