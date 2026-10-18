<template>
	<div
		class="flex-1 h-full overflow-auto bg-white dark:bg-neutral-900 select-none relative focus:outline-none"
		tabindex="0"
		@click="clearSelection"
		@contextmenu.prevent="handleContextMenu($event, null)"
		@keydown="handleKeyNavigation"
	>
		<!-- Loading state -->
		<div v-if="isLoading || searchLoading" class="absolute inset-0 bg-white/50 dark:bg-neutral-900/50 backdrop-blur-xs flex items-center justify-center z-10">
			<div class="flex items-center gap-2 px-4 py-2 rounded-lg bg-white dark:bg-neutral-800 shadow-md border border-neutral-200 dark:border-neutral-700 text-xs">
				<UIcon name="lucide:loader-2" class="w-4 h-4 animate-spin text-primary-500" />
				<span>{{ isSearching ? 'Searching with tgrep...' : 'Loading folder...' }}</span>
			</div>
		</div>

		<!-- Search results view (tgrep) -->
		<div v-if="isSearching" class="p-4">
			<div class="flex items-center justify-between pb-3 mb-3 border-b border-neutral-200 dark:border-neutral-800">
				<div class="flex items-center gap-2">
					<UIcon name="lucide:zap" class="w-4 h-4 text-amber-500" />
					<span class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
						tgrep results for "{{ searchQuery }}"
					</span>
					<span class="text-xs px-2 py-0.5 rounded-full bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400">
						{{ searchHits.length }} matches
					</span>
				</div>
				<UButton variant="ghost" size="xs" icon="lucide:x" @click="cancelSearch">
					Close search
				</UButton>
			</div>

			<!-- Search hits list -->
			<div v-if="searchHits.length > 0" class="space-y-1">
				<div
					v-for="(hit, idx) in searchHits"
					:key="idx"
					class="p-2.5 rounded-lg border border-neutral-200/80 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-800/60 cursor-pointer transition-colors"
					@click.stop="openSearchHit(hit)"
				>
					<div class="flex items-center gap-2">
						<ExplorerFileIcon :path="hit.path" :is-dir="hit.is_dir" :fallback="getFileIcon(hit.name, hit.is_dir)" class="w-4 h-4" />
						<span class="font-medium text-xs text-neutral-900 dark:text-neutral-100 truncate">{{ hit.name }}</span>
						<span v-if="hit.line_number" class="text-[11px] font-mono px-1.5 py-0.2 rounded bg-primary-100 dark:bg-primary-950/60 text-primary-700 dark:text-primary-300">
							Line {{ hit.line_number }}
						</span>
					</div>

					<div class="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono truncate mt-0.5">
						{{ hit.path }}
					</div>

					<!-- Matching line snippet if content search -->
					<div
						v-if="hit.line_content"
						class="mt-1.5 p-1.5 rounded bg-neutral-100 dark:bg-neutral-950 font-mono text-[11px] text-neutral-800 dark:text-neutral-200 border-l-2 border-primary-500 overflow-x-auto whitespace-pre"
					>
						{{ hit.line_content }}
					</div>
				</div>
			</div>

			<div v-else-if="!searchLoading" class="py-12 text-center text-neutral-400 text-xs">
				No results found for "{{ searchQuery }}". Try toggling content search mode or case sensitivity.
			</div>
		</div>

		<!-- Normal Folder View -->
		<div v-else class="h-full">
			<!-- Details View Mode -->
			<div v-if="viewMode === 'details'" class="min-w-full">
				<!-- Table Header -->
				<div class="flex items-center h-8 px-3 border-b border-neutral-200 dark:border-neutral-800 text-[11px] font-semibold text-neutral-500 dark:text-neutral-400 sticky top-0 bg-neutral-50 dark:bg-neutral-900 z-5 select-none">
					<div class="w-8 shrink-0 flex items-center justify-center">
						<input
							type="checkbox"
							:checked="selectedCount > 0 && selectedCount === items.length"
							class="rounded text-primary-600 focus:ring-0 cursor-pointer"
							@change="toggleSelectAll"
						/>
					</div>
					<div class="flex-1 flex items-center gap-1 cursor-pointer hover:text-neutral-900 dark:hover:text-neutral-200" @click="handleSort('name')">
						<span>Name</span>
						<UIcon v-if="sortBy === 'name'" :name="sortOrder === 'asc' ? 'lucide:arrow-up' : 'lucide:arrow-down'" class="w-3 h-3" />
					</div>
					<div class="w-40 shrink-0 flex items-center gap-1 cursor-pointer hover:text-neutral-900 dark:hover:text-neutral-200 hidden md:flex" @click="handleSort('modified')">
						<span>Date modified</span>
						<UIcon v-if="sortBy === 'modified'" :name="sortOrder === 'asc' ? 'lucide:arrow-up' : 'lucide:arrow-down'" class="w-3 h-3" />
					</div>
					<div class="w-28 shrink-0 flex items-center gap-1 cursor-pointer hover:text-neutral-900 dark:hover:text-neutral-200 hidden lg:flex" @click="handleSort('type')">
						<span>Type</span>
						<UIcon v-if="sortBy === 'type'" :name="sortOrder === 'asc' ? 'lucide:arrow-up' : 'lucide:arrow-down'" class="w-3 h-3" />
					</div>
					<div class="w-24 shrink-0 flex items-center justify-end gap-1 cursor-pointer hover:text-neutral-900 dark:hover:text-neutral-200" @click="handleSort('size')">
						<span>Size</span>
						<UIcon v-if="sortBy === 'size'" :name="sortOrder === 'asc' ? 'lucide:arrow-up' : 'lucide:arrow-down'" class="w-3 h-3" />
					</div>
				</div>

				<!-- Items Rows -->
				<div class="divide-y divide-neutral-100 dark:divide-neutral-800/40 text-xs">
					<div
						v-for="item in sortedItems"
						:key="item.path"
						class="flex items-center h-8 px-3 cursor-pointer transition-colors"
						:class="[
							selectedPaths.has(item.path)
								? 'bg-blue-50 dark:bg-blue-950/40 text-blue-900 dark:text-blue-100 font-medium'
								: 'hover:bg-neutral-100/70 dark:hover:bg-neutral-800/50 text-neutral-800 dark:text-neutral-200'
						]"
						@click.stop="handleRowClick($event, item)"
						@dblclick.stop="openItem(item)"
						@contextmenu.prevent.stop="handleContextMenu($event, item)"
					>
						<!-- Checkbox -->
						<div class="w-8 shrink-0 flex items-center justify-center">
							<input
								type="checkbox"
								:checked="selectedPaths.has(item.path)"
								class="rounded text-primary-600 focus:ring-0 cursor-pointer"
								@click.stop="selectItem(item.path, true)"
							/>
						</div>

						<!-- Name + Icon -->
						<div class="flex-1 flex items-center gap-2 min-w-0 pr-2">
							<ExplorerFileIcon :path="item.path" :is-dir="item.is_dir" :fallback="getFileIcon(item.name, item.is_dir)" class="w-4 h-4" />
							<span class="truncate">{{ item.name }}</span>
							<span
								v-if="getGitFileBadge(item)"
								class="px-1.5 py-0.2 rounded text-[10px] font-mono font-bold shrink-0"
								:class="getGitFileBadge(item)!.class"
								:title="getGitFileBadge(item)!.title"
							>
								{{ getGitFileBadge(item)!.label }}
							</span>
						</div>

						<!-- Date modified -->
						<div class="w-40 shrink-0 text-neutral-500 dark:text-neutral-400 text-[11px] truncate hidden md:block">
							{{ formatDate(item.modified) }}
						</div>

						<!-- Type -->
						<div class="w-28 shrink-0 text-neutral-500 dark:text-neutral-400 text-[11px] truncate hidden lg:block">
							{{ item.is_dir ? 'File folder' : getFileTypeLabel(item.extension) }}
						</div>

						<!-- Size -->
						<div class="w-24 shrink-0 text-right text-neutral-500 dark:text-neutral-400 text-[11px] truncate">
							{{ item.is_dir ? '' : formatBytes(item.size) }}
						</div>
					</div>
				</div>
			</div>

			<!-- Large Icons Mode -->
			<div v-else-if="viewMode === 'large'" class="p-4 grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3">
				<div
					v-for="item in sortedItems"
					:key="item.path"
					class="group flex flex-col items-center justify-center p-3 rounded-lg border border-transparent transition-all cursor-pointer text-center"
					:class="[
						selectedPaths.has(item.path)
							? 'bg-blue-50 dark:bg-blue-950/50 border-blue-200 dark:border-blue-800 text-blue-900 dark:text-blue-100 shadow-xs'
							: 'hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-800 dark:text-neutral-200'
					]"
					@click.stop="handleRowClick($event, item)"
					@dblclick.stop="openItem(item)"
					@contextmenu.prevent.stop="handleContextMenu($event, item)"
				>
					<ExplorerFileIcon
						:path="item.path"
						:is-dir="item.is_dir"
						:fallback="getFileIcon(item.name, item.is_dir)"
						class="w-12 h-12 mb-2 transition-transform group-hover:scale-105"
					/>
					<span class="text-xs font-medium line-clamp-2 break-all w-full leading-tight">
						{{ item.name }}
					</span>
					<span v-if="!item.is_dir" class="text-[10px] text-neutral-400 mt-1">
						{{ formatBytes(item.size) }}
					</span>
				</div>
			</div>

			<!-- Grid / Tiles Mode -->
			<div v-else class="p-3 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
				<div
					v-for="item in sortedItems"
					:key="item.path"
					class="flex items-center gap-3 p-2 rounded-lg border border-neutral-100 dark:border-neutral-800 transition-colors cursor-pointer"
					:class="[
						selectedPaths.has(item.path)
							? 'bg-blue-50 dark:bg-blue-950/50 border-blue-200 dark:border-blue-800 text-blue-900 dark:text-blue-100 shadow-xs'
							: 'hover:bg-neutral-100/70 dark:hover:bg-neutral-800/60 text-neutral-800 dark:text-neutral-200'
					]"
					@click.stop="handleRowClick($event, item)"
					@dblclick.stop="openItem(item)"
					@contextmenu.prevent.stop="handleContextMenu($event, item)"
				>
					<ExplorerFileIcon
						:path="item.path"
						:is-dir="item.is_dir"
						:fallback="getFileIcon(item.name, item.is_dir)"
						class="w-8 h-8"
					/>
					<div class="min-w-0 flex-1 text-xs">
						<div class="font-medium truncate">{{ item.name }}</div>
						<div class="text-[11px] text-neutral-400">
							{{ item.is_dir ? 'File folder' : formatBytes(item.size) }}
						</div>
					</div>
				</div>
			</div>

			<!-- Empty folder notice -->
			<div v-if="items.length === 0 && !isLoading" class="flex flex-col items-center justify-center py-20 text-neutral-400">
				<UIcon name="lucide:folder-open" class="w-12 h-12 stroke-[1.2] mb-2 text-neutral-300 dark:text-neutral-700" />
				<span class="text-xs">This folder is empty.</span>
			</div>
		</div>

		<!-- Context Menu Component -->
		<ExplorerContextMenu
			v-if="contextMenuVisible"
			:x="contextMenuX"
			:y="contextMenuY"
			:target-item="contextMenuItem"
			@close="contextMenuVisible = false"
		/>
	</div>
</template>

<script lang="ts" setup>
	import type { FileItem, SearchHit } from '~/composables/useExplorerApi';

	const {
		items,
		sortedItems,
		selectedPaths,
		selectedCount,
		viewMode,
		sortBy,
		sortOrder,
		isLoading,
		isSearching,
		searchHits,
		searchLoading,
		searchQuery,
		cancelSearch,
		selectItem,
		selectAll,
		clearSelection,
		openItem,
		loadDirectory,
		togglePeek,
		navigateSelection,
		gitStatus
	} = useExplorer();

	const api = useExplorerApi();

	function getGitFileBadge(item: FileItem) {
		if (!gitStatus.value?.fileStatuses) return null;
		const code = gitStatus.value.fileStatuses[item.path] || gitStatus.value.fileStatuses[item.name];
		if (!code) return null;

		if (code.includes('M')) {
			return { label: 'M', title: 'Modified', class: 'bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-400' };
		}
		if (code.includes('A')) {
			return { label: 'A', title: 'Added (Staged)', class: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-400' };
		}
		if (code.includes('?') || code.includes('U')) {
			return { label: 'U', title: 'Untracked', class: 'bg-blue-100 text-blue-700 dark:bg-blue-950/60 dark:text-blue-400' };
		}
		if (code.includes('D')) {
			return { label: 'D', title: 'Deleted', class: 'bg-rose-100 text-rose-700 dark:bg-rose-950/60 dark:text-rose-400' };
		}
		return { label: code, title: code, class: 'bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-300' };
	}

	// Context menu state
	const contextMenuVisible = ref(false);
	const contextMenuX = ref(0);
	const contextMenuY = ref(0);
	const contextMenuItem = ref<FileItem | null>(null);

	const handleRowClick = (e: MouseEvent, item: FileItem) => {
		if (e.ctrlKey || e.metaKey) {
			selectItem(item.path, true);
		} else {
			selectItem(item.path, false);
		}
	};

	const toggleSelectAll = () => {
		if (selectedCount.value === items.value.length) {
			clearSelection();
		} else {
			selectAll();
		}
	};

	const handleSort = (field: 'name' | 'modified' | 'type' | 'size') => {
		if (sortBy.value === field) {
			sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy.value = field;
			sortOrder.value = 'asc';
		}
	};

	const handleContextMenu = (e: MouseEvent, item: FileItem | null) => {
		if (item && !selectedPaths.value.has(item.path)) {
			selectItem(item.path, false);
		}
		contextMenuItem.value = item;
		contextMenuX.value = e.clientX;
		contextMenuY.value = e.clientY;
		contextMenuVisible.value = true;
	};

	const openSearchHit = (hit: SearchHit) => {
		if (hit.is_dir) {
			loadDirectory(hit.path);
		} else {
			api.executeAction('open', hit.path);
		}
	};

	const handleKeyNavigation = (e: KeyboardEvent) => {
		const targetTag = (e.target as HTMLElement)?.tagName?.toLowerCase();
		if (targetTag === 'input' || targetTag === 'textarea') return;

		if (e.key === 'a' && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			selectAll();
		} else if (e.code === 'Space') {
			e.preventDefault();
			togglePeek();
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			navigateSelection('up');
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			navigateSelection('down');
		} else if (e.key === 'Enter' && !e.altKey) { // Ctrl+M / Ctrl+Shift+V / Ctrl+P / Alt+Enter: global handler in pages/index.vue
			if (selectedCount.value === 1) {
				const target = items.value.find(i => selectedPaths.value.has(i.path));
				if (target) {
					openItem(target);
				}
			}
		} else if (e.key === 'Escape') {
			clearSelection();
		}
	};

	// Helper icon mapper
	const getFileIcon = (name: string, isDir: boolean) => {
		if (isDir) return 'lucide:folder';
		const ext = name.split('.').pop()?.toLowerCase() || '';

		switch (ext) {
			case 'png':
			case 'jpg':
			case 'jpeg':
			case 'webp':
			case 'gif':
			case 'svg':
			case 'ico':
				return 'lucide:image';
			case 'mp4':
			case 'mkv':
			case 'mov':
			case 'avi':
				return 'lucide:video';
			case 'mp3':
			case 'wav':
			case 'ogg':
			case 'flac':
				return 'lucide:music';
			case 'zip':
			case 'rar':
			case '7z':
			case 'tar':
			case 'gz':
				return 'lucide:archive';
			case 'ts':
			case 'js':
			case 'vue':
			case 'rs':
			case 'py':
			case 'html':
			case 'css':
			case 'json':
				return 'lucide:file-code-2';
			case 'exe':
			case 'msi':
			case 'bat':
			case 'cmd':
				return 'lucide:terminal';
			case 'pdf':
				return 'lucide:file-text';
			default:
				return 'lucide:file';
		}
	};

	const getFileTypeLabel = (ext: string) => {
		if (!ext) return 'File';
		return `${ext.toUpperCase()} File`;
	};

	const formatBytes = (bytes: number) => {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
	};

	const formatDate = (ms: number) => {
		if (!ms) return '';
		const d = new Date(ms);
		return `${d.toLocaleDateString()} ${d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
	};
</script>
