import type { FileItem, DriveItem, QuickAccessItem, SearchHit, FilePreview, TreeSizeResult, FileHashResult, GitStatusResult } from './useExplorerApi';

export interface ExplorerTab {
	id: string;
	title: string;
	path: string;
	history: string[];
	historyIndex: number;
}

export type ViewMode = 'details' | 'large' | 'grid' | 'tiles';
export type SortField = 'name' | 'modified' | 'type' | 'size';

// Shared Global State
const tabs = ref<ExplorerTab[]>([
	{
		id: 'tab-1',
		title: 'Home',
		path: '',
		history: [],
		historyIndex: 0
	}
]);
const activeTabId = ref<string>('tab-1');
const currentPath = ref<string>('');
const items = ref<FileItem[]>([]);
const drives = ref<DriveItem[]>([]);
const quickAccess = ref<QuickAccessItem[]>([]);
const isLoading = ref<boolean>(false);
const error = ref<string | null>(null);

const selectedPaths = ref<Set<string>>(new Set());
const viewMode = ref<ViewMode>('details');
const showHidden = ref<boolean>(false);
const showPreviewPane = ref<boolean>(false);

const sortBy = ref<SortField>('name');
const sortOrder = ref<'asc' | 'desc'>('asc');

const searchQuery = ref<string>('');
const isSearching = ref<boolean>(false);
const searchHits = ref<SearchHit[]>([]);
const searchLoading = ref<boolean>(false);
const contentSearch = ref<boolean>(false);
const searchCaseSensitive = ref<boolean>(false);
const searchRegex = ref<boolean>(false);

const clipboard = ref<{ paths: string[]; action: 'cut' | 'copy' } | null>(null);
const activePreview = ref<FilePreview | null>(null);
const previewLoading = ref<boolean>(false);

// PowerToys Modals & Features
const isPeekOpen = ref<boolean>(false);
const isPowerRenameOpen = ref<boolean>(false);
const isCommandPaletteOpen = ref<boolean>(false);
const renameModalOpen = ref<boolean>(false);
const renameTarget = ref<FileItem | null>(null);
const newFolderModalOpen = ref<boolean>(false);
const propertiesModalOpen = ref<boolean>(false);
const propertiesTarget = ref<FileItem | null>(null);

// New Features: TreeSize, Hasher, Image Resizer, Dual Pane, Git Status
const isTreeSizeOpen = ref<boolean>(false);
const treeSizeData = ref<TreeSizeResult | null>(null);
const treeSizeLoading = ref<boolean>(false);
const treeSizeProgress = ref<number>(0);

const isHashModalOpen = ref<boolean>(false);
const hashTarget = ref<FileItem | null>(null);
const hashResult = ref<FileHashResult | null>(null);
const hashLoading = ref<boolean>(false);

const isImageResizerOpen = ref<boolean>(false);
const imageResizerTarget = ref<FileItem | null>(null);

const isDualPane = ref<boolean>(false);
const dualPanePath = ref<string>('');
const dualPaneItems = ref<FileItem[]>([]);
const dualPaneSelectedPaths = ref<Set<string>>(new Set());
const activePane = ref<'primary' | 'secondary'>('primary');

const gitStatus = ref<GitStatusResult | null>(null);

export const useExplorer = () => {
	const api = useExplorerApi();

	const activeTab = computed(() => {
		return tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0];
	});

	// Computed sorted items
	const sortedItems = computed(() => {
		const list = [...items.value];
		list.sort((a, b) => {
			if (a.is_dir !== b.is_dir) {
				return a.is_dir ? -1 : 1;
			}

			let cmp = 0;
			switch (sortBy.value) {
				case 'name':
					cmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
					break;
				case 'modified':
					cmp = a.modified - b.modified;
					break;
				case 'type':
					cmp = a.extension.localeCompare(b.extension);
					break;
				case 'size':
					cmp = a.size - b.size;
					break;
			}

			return sortOrder.value === 'asc' ? cmp : -cmp;
		});

		return list;
	});

	// Selection counts
	const selectedCount = computed(() => selectedPaths.value.size);
	const selectedSize = computed(() => {
		let total = 0;
		for (const path of selectedPaths.value) {
			const item = items.value.find(i => i.path === path);
			if (item && !item.is_dir) {
				total += item.size;
			}
		}
		return total;
	});

	// Load directory
	const loadDirectory = async (targetPath?: string, recordHistory = true) => {
		isLoading.value = true;
		error.value = null;
		selectedPaths.value.clear();

		// Exit search mode
		isSearching.value = false;
		searchQuery.value = '';
		searchHits.value = [];

		try {
			// If no path, get home from quick access or drives
			let p = targetPath || activeTab.value.path;
			if (!p) {
				if (quickAccess.value.length > 0) {
					p = quickAccess.value[0].path;
				} else if (drives.value.length > 0) {
					p = drives.value[0].path;
				} else {
					p = 'C:\\';
				}
			}

			const res = await api.listDirectory(p, showHidden.value);
			currentPath.value = res.current_path;
			items.value = res.items;

			// Fetch Git status in background
			fetchGitStatus(res.current_path);

			// Update active tab title and history
			const tab = activeTab.value;
			tab.path = res.current_path;
			tab.title = getPathDisplayName(res.current_path);

			if (recordHistory) {
				if (tab.historyIndex < tab.history.length - 1) {
					tab.history = tab.history.slice(0, tab.historyIndex + 1);
				}
				tab.history.push(res.current_path);
				tab.historyIndex = tab.history.length - 1;
			}

			// Update preview if preview pane is open
			if (showPreviewPane.value && items.value.length > 0) {
				selectItem(items.value[0].path, false);
			}
		} catch (err: any) {
			error.value = err?.message || 'Failed to open directory';
		} finally {
			isLoading.value = false;
		}
	};

	const getPathDisplayName = (p: string) => {
		if (!p) return 'This PC';
		const drive = drives.value.find(d => d.path.toLowerCase() === p.toLowerCase() || d.letter.toLowerCase() === p.toLowerCase());
		if (drive) return drive.label;
		const qa = quickAccess.value.find(q => q.path.toLowerCase() === p.toLowerCase());
		if (qa) return qa.name;
		const parts = p.split(/[\\/]/).filter(Boolean);
		return parts[parts.length - 1] || p;
	};

	// Navigation actions
	const canGoBack = computed(() => activeTab.value.historyIndex > 0);
	const canGoForward = computed(() => activeTab.value.historyIndex < activeTab.value.history.length - 1);

	const goBack = () => {
		if (!canGoBack.value) return;
		activeTab.value.historyIndex--;
		const prev = activeTab.value.history[activeTab.value.historyIndex];
		loadDirectory(prev, false);
	};

	const goForward = () => {
		if (!canGoForward.value) return;
		activeTab.value.historyIndex++;
		const next = activeTab.value.history[activeTab.value.historyIndex];
		loadDirectory(next, false);
	};

	const goUp = () => {
		const p = currentPath.value;
		if (!p) return;
		// If drive root (e.g. C:\ or C:), go to Home / This PC
		if (/^[a-zA-Z]:[\\/]?$/.test(p)) {
			// Already top
			return;
		}
		const parent = p.replace(/[\\/][^\\/]+[\\/]?$/, '');
		const finalParent = /^[a-zA-Z]:$/.test(parent) ? `${parent}\\` : parent;
		loadDirectory(finalParent);
	};

	const refresh = () => {
		loadDirectory(currentPath.value, false);
	};

	// Tabs management
	const addTab = (path?: string) => {
		const newPath = path || currentPath.value;
		const id = `tab-${Date.now()}`;
		tabs.value.push({
			id,
			title: getPathDisplayName(newPath),
			path: newPath,
			history: [newPath],
			historyIndex: 0
		});
		activeTabId.value = id;
		loadDirectory(newPath, false);
	};

	const closeTab = (id: string) => {
		if (tabs.value.length <= 1) return;
		const index = tabs.value.findIndex(t => t.id === id);
		tabs.value = tabs.value.filter(t => t.id !== id);
		if (activeTabId.value === id) {
			const nextIndex = Math.max(0, index - 1);
			activeTabId.value = tabs.value[nextIndex].id;
			loadDirectory(tabs.value[nextIndex].path, false);
		}
	};

	const switchTab = (id: string) => {
		activeTabId.value = id;
		const tab = tabs.value.find(t => t.id === id);
		if (tab) {
			loadDirectory(tab.path, false);
		}
	};

	// Selection
	const selectItem = (path: string, isMulti = false, isRange = false) => {
		if (!isMulti && !isRange) {
			selectedPaths.value.clear();
			selectedPaths.value.add(path);
		} else if (isMulti) {
			if (selectedPaths.value.has(path)) {
				selectedPaths.value.delete(path);
			} else {
				selectedPaths.value.add(path);
			}
		}

		// Trigger preview if single item selected
		if (selectedPaths.value.size === 1) {
			loadPreview(Array.from(selectedPaths.value)[0]);
		} else {
			activePreview.value = null;
		}
	};

	const selectAll = () => {
		selectedPaths.value = new Set(items.value.map(i => i.path));
	};

	const clearSelection = () => {
		selectedPaths.value.clear();
		activePreview.value = null;
	};

	const selectedItem = computed(() => {
		if (selectedPaths.value.size === 1) {
			const targetPath = Array.from(selectedPaths.value)[0];
			return items.value.find(i => i.path === targetPath) || null;
		}
		return null;
	});

	// PowerToys Peek Navigation (ArrowUp/ArrowDown)
	const navigateSelection = (direction: 'up' | 'down') => {
		if (sortedItems.value.length === 0) return;
		const list = sortedItems.value;
		if (selectedPaths.value.size === 0) {
			const first = list[0];
			selectedPaths.value = new Set([first.path]);
			loadPreview(first.path);
			return;
		}

		const currentSelectedPath = Array.from(selectedPaths.value)[0];
		const currentIndex = list.findIndex(i => i.path === currentSelectedPath);
		let nextIndex = currentIndex;

		if (direction === 'up') {
			nextIndex = currentIndex > 0 ? currentIndex - 1 : list.length - 1;
		} else {
			nextIndex = currentIndex < list.length - 1 ? currentIndex + 1 : 0;
		}

		const nextItem = list[nextIndex];
		if (nextItem) {
			selectedPaths.value = new Set([nextItem.path]);
			loadPreview(nextItem.path);
		}
	};

	// PowerToys Spacebar Peek Toggle
	const togglePeek = () => {
		if (isPeekOpen.value) {
			isPeekOpen.value = false;
			return;
		}
		if (selectedPaths.value.size === 0 && sortedItems.value.length > 0) {
			selectedPaths.value = new Set([sortedItems.value[0].path]);
		}
		if (selectedPaths.value.size > 0) {
			const target = Array.from(selectedPaths.value)[0];
			loadPreview(target);
			isPeekOpen.value = true;
		}
	};

	// PowerToys PowerRename
	const openPowerRename = () => {
		isPowerRenameOpen.value = true;
	};

	const batchRename = async (renames: { oldPath: string; newPath: string }[]) => {
		isLoading.value = true;
		try {
			for (const r of renames) {
				await api.executeAction('rename', r.oldPath, { newPath: r.newPath });
			}
			await loadDirectory(currentPath.value, false);
		} catch (e) {
			console.error('Failed to batch rename', e);
		} finally {
			isLoading.value = false;
			isPowerRenameOpen.value = false;
		}
	};

	// Explorer parity: Delete -> Recycle Bin without prompt, Shift+Delete -> confirm + permanent.
	const deletePaths = async (paths: string[], permanent = false) => {
		if (!paths.length) return false;
		if (permanent && !confirm(`Permanently delete ${paths.length} item(s)? This cannot be undone.`)) return false;
		for (const p of paths) {
			await api.executeAction(permanent ? 'deletePermanent' : 'delete', p);
		}
		selectedPaths.value.clear();
		refresh();
		return true;
	};

	// PowerToys Paste as File (Ctrl + Shift + V)
	const pasteAsFile = async () => {
		if (!currentPath.value) return;
		try {
			if (typeof navigator !== 'undefined' && navigator.clipboard?.read) {
				try {
					const clipItems = await navigator.clipboard.read();
					for (const clipItem of clipItems) {
						const imageType = clipItem.types.find(t => t.startsWith('image/'));
						if (imageType) {
							const blob = await clipItem.getType(imageType);
							const reader = new FileReader();
							reader.onloadend = async () => {
								const base64 = (reader.result as string)?.split(',')[1];
								const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
								const ext = imageType.split('/')[1] || 'png';
								await api.executeAction('createFile', currentPath.value, {
									name: `Pasted_Image_${timestamp}.${ext}`,
									contentBase64: base64
								});
								await loadDirectory(currentPath.value, false);
							};
							reader.readAsDataURL(blob);
							return;
						}
					}
				} catch {
					// Fallback to text
				}
			}

			if (typeof navigator !== 'undefined' && navigator.clipboard?.readText) {
				const text = await navigator.clipboard.readText();
				if (text && text.trim()) {
					const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
					let ext = 'txt';
					const trimmed = text.trim();
					if (trimmed.startsWith('{') || trimmed.startsWith('[')) ext = 'json';
					else if (trimmed.includes('import ') || trimmed.includes('export ') || trimmed.includes('function ')) ext = 'ts';
					else if (trimmed.startsWith('# ') || trimmed.includes('## ')) ext = 'md';

					await api.executeAction('createFile', currentPath.value, {
						name: `Pasted_Snippet_${timestamp}.${ext}`,
						content: text
					});
					await loadDirectory(currentPath.value, false);
				}
			}
		} catch (err) {
			console.error('Error pasting as file', err);
		}
	};

	const toggleCommandPalette = () => {
		isCommandPaletteOpen.value = !isCommandPaletteOpen.value;
	};

	// Preview loader
	const loadPreview = async (path: string) => {
		previewLoading.value = true;
		try {
			activePreview.value = await api.getPreview(path);
		} catch {
			activePreview.value = null;
		} finally {
			previewLoading.value = false;
		}
	};

	// Open item
	const openItem = async (item: FileItem) => {
		if (item.is_dir) {
			loadDirectory(item.path);
		} else {
			await api.executeAction('open', item.path);
		}
	};

	// Search with tgrep
	const triggerSearch = async (query: string) => {
		if (!query.trim()) {
			isSearching.value = false;
			searchHits.value = [];
			return;
		}

		isSearching.value = true;
		searchLoading.value = true;
		searchQuery.value = query;

		try {
			// Filename search: in-memory index first (instant), tgrep while it is still building.
			if (!contentSearch.value && !searchRegex.value) {
				const hits = await api.searchNames(query, currentPath.value);
				if (hits) {
					searchHits.value = hits;
					return;
				}
			}
			searchHits.value = await api.runTgrep(
				query,
				currentPath.value,
				contentSearch.value,
				searchCaseSensitive.value,
				searchRegex.value
			);
		} catch {
			searchHits.value = [];
		} finally {
			searchLoading.value = false;
		}
	};

	const cancelSearch = () => {
		isSearching.value = false;
		searchQuery.value = '';
		searchHits.value = [];
	};

	// Git Status
	const fetchGitStatus = async (p?: string) => {
		const target = p || currentPath.value;
		if (!target) return;
		try {
			const res = await api.getGitStatus(target);
			gitStatus.value = res;
		} catch {
			gitStatus.value = null;
		}
	};

	// TreeSize Storage Analyzer
	const openTreeSize = async (targetFolder?: string, rescan = false) => {
		const target = targetFolder || currentPath.value;
		if (!target) return;
		isTreeSizeOpen.value = true;
		treeSizeLoading.value = true;
		treeSizeProgress.value = 0;
		const { listen } = await import('@tauri-apps/api/event');
		const unlisten = await listen<number>('treesize-progress', e => (treeSizeProgress.value = e.payload));
		try {
			treeSizeData.value = await api.getTreeSize(target, rescan);
		} catch (err: any) {
			console.error('Failed to load TreeSize:', err);
		} finally {
			unlisten();
			treeSizeLoading.value = false;
		}
	};

	// File Checksum / Hasher
	const openHashModal = async (item?: FileItem) => {
		const target = item || selectedItem.value;
		if (!target || target.is_dir) return;
		hashTarget.value = target;
		isHashModalOpen.value = true;
		hashLoading.value = true;
		hashResult.value = null;
		try {
			const res = await api.getFileHash(target.path);
			hashResult.value = res;
		} catch (err: any) {
			console.error('Failed to compute hash:', err);
		} finally {
			hashLoading.value = false;
		}
	};

	// Image Resizer & Converter
	const openImageResizer = (item?: FileItem) => {
		const target = item || selectedItem.value;
		if (!target || target.is_dir) return;
		imageResizerTarget.value = target;
		isImageResizerOpen.value = true;
	};

	// Dual Pane (Split View)
	const toggleDualPane = async () => {
		isDualPane.value = !isDualPane.value;
		if (isDualPane.value && !dualPanePath.value) {
			dualPanePath.value = currentPath.value;
			await loadDualPaneDirectory(dualPanePath.value);
		}
	};

	const loadDualPaneDirectory = async (p: string) => {
		try {
			const res = await api.listDirectory(p, showHidden.value);
			dualPanePath.value = res.current_path;
			dualPaneItems.value = res.items;
		} catch (err: any) {
			console.error('Failed to load dual pane directory:', err);
		}
	};

	// Initialize system drives & quick access
	const initialize = async () => {
		try {
			const [d, qa] = await Promise.all([
				api.getDrives(),
				api.getQuickAccess()
			]);
			drives.value = d;
			quickAccess.value = qa;

			// Load initial directory
			const initialPath = qa[0]?.path || d[0]?.path || 'C:\\';
			loadDirectory(initialPath);
		} catch {
			loadDirectory('C:\\');
		}
	};

	return {
		// State
		tabs,
		activeTabId,
		activeTab,
		currentPath,
		items,
		sortedItems,
		drives,
		quickAccess,
		isLoading,
		error,
		selectedPaths,
		selectedCount,
		selectedSize,
		viewMode,
		showHidden,
		showPreviewPane,
		sortBy,
		sortOrder,
		clipboard,
		activePreview,
		previewLoading,

		// Search
		searchQuery,
		isSearching,
		searchHits,
		searchLoading,
		contentSearch,
		searchCaseSensitive,
		searchRegex,

		// Modals & PowerToys
		renameModalOpen,
		renameTarget,
		newFolderModalOpen,
		propertiesModalOpen,
		propertiesTarget,
		isPeekOpen,
		isPowerRenameOpen,
		isCommandPaletteOpen,
		selectedItem,

		// TreeSize & Advanced Features
		isTreeSizeOpen,
		treeSizeData,
		treeSizeLoading,
		treeSizeProgress,
		isHashModalOpen,
		hashTarget,
		hashResult,
		hashLoading,
		isImageResizerOpen,
		imageResizerTarget,
		isDualPane,
		dualPanePath,
		dualPaneItems,
		dualPaneSelectedPaths,
		activePane,
		gitStatus,

		// Methods
		initialize,
		loadDirectory,
		goBack,
		goForward,
		goUp,
		refresh,
		canGoBack,
		canGoForward,
		addTab,
		closeTab,
		switchTab,
		selectItem,
		selectAll,
		clearSelection,
		openItem,
		triggerSearch,
		cancelSearch,
		loadPreview,
		getPathDisplayName,
		togglePeek,
		navigateSelection,
		openPowerRename,
		batchRename,
		deletePaths,
		pasteAsFile,
		toggleCommandPalette,
		openTreeSize,
		openHashModal,
		openImageResizer,
		toggleDualPane,
		loadDualPaneDirectory,
		fetchGitStatus
	};
};
