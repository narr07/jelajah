<template>
	<div class="flex items-center gap-2 px-3 py-1.5 bg-neutral-50/80 dark:bg-neutral-900/80 backdrop-blur border-b border-neutral-200 dark:border-neutral-800">
		<!-- History Navigation Buttons -->
		<div class="flex items-center gap-0.5">
			<button
				class="p-1.5 rounded-md transition-colors"
				:class="canGoBack ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="!canGoBack"
				title="Back (Alt+Left)"
				@click="goBack"
			>
				<UIcon name="lucide:arrow-left" class="w-4 h-4" />
			</button>

			<button
				class="p-1.5 rounded-md transition-colors"
				:class="canGoForward ? 'text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800' : 'text-neutral-300 dark:text-neutral-600 cursor-not-allowed'"
				:disabled="!canGoForward"
				title="Forward (Alt+Right)"
				@click="goForward"
			>
				<UIcon name="lucide:arrow-right" class="w-4 h-4" />
			</button>

			<button
				class="p-1.5 rounded-md text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
				title="Up to parent folder (Alt+Up)"
				@click="goUp"
			>
				<UIcon name="lucide:arrow-up" class="w-4 h-4" />
			</button>

			<button
				class="p-1.5 rounded-md text-neutral-700 dark:text-neutral-200 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
				title="Refresh (F5)"
				@click="refresh"
			>
				<UIcon name="lucide:rotate-cw" class="w-4 h-4" :class="{ 'animate-spin': isLoading }" />
			</button>
		</div>

		<!-- Breadcrumb Address Bar -->
		<div
			class="flex-1 flex items-center h-8 px-2.5 rounded-md bg-white dark:bg-neutral-800/80 border border-neutral-200 dark:border-neutral-700/80 text-xs shadow-xs focus-within:ring-2 focus-within:ring-primary-500/20 focus-within:border-primary-500 transition-all cursor-text overflow-hidden"
			@click="enableEditMode"
		>
			<UIcon name="lucide:folder" class="w-4 h-4 text-amber-500 mr-2 shrink-0" />

			<!-- Text Input Mode -->
			<form v-if="isEditingPath" class="flex-1 flex items-center" @submit.prevent="submitPath">
				<input
					ref="pathInputRef"
					v-model="inputPath"
					type="text"
					class="w-full bg-transparent outline-none text-neutral-900 dark:text-neutral-100 font-mono text-xs"
					@blur="disableEditMode"
					@keydown.esc="disableEditMode"
				/>
			</form>

			<!-- Segmented Breadcrumb Mode -->
			<div v-else class="flex items-center gap-0.5 overflow-x-auto no-scrollbar flex-1">
				<div
					v-for="(segment, idx) in pathSegments"
					:key="idx"
					class="flex items-center shrink-0"
				>
					<button
						type="button"
						class="px-1.5 py-0.5 rounded hover:bg-neutral-200/70 dark:hover:bg-neutral-700/70 text-neutral-700 dark:text-neutral-300 transition-colors font-medium"
						@click.stop="loadDirectory(segment.path)"
					>
						{{ segment.name }}
					</button>
					<UIcon
						v-if="idx < pathSegments.length - 1"
						name="lucide:chevron-right"
						class="w-3.5 h-3.5 text-neutral-400 shrink-0 mx-0.5"
					/>
				</div>
			</div>

			<!-- Copy Path Button -->
			<button
				type="button"
				class="p-1 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-700 shrink-0 ml-1 transition-colors"
				title="Copy path (Ctrl+Shift+C)"
				@click.stop="copyCurrentPath"
			>
				<UIcon :name="copied ? 'lucide:check' : 'lucide:copy'" class="w-3.5 h-3.5 text-neutral-500" />
			</button>
		</div>

		<!-- Fast Search Bar with tgrep -->
		<div class="relative w-64 lg:w-80">
			<div class="flex items-center h-8 px-2.5 rounded-md bg-white dark:bg-neutral-800/80 border border-neutral-200 dark:border-neutral-700/80 text-xs shadow-xs focus-within:ring-2 focus-within:ring-primary-500/20 focus-within:border-primary-500 transition-all">
				<UIcon name="lucide:search" class="w-3.5 h-3.5 text-neutral-400 mr-2 shrink-0" />
				<input
					v-model="searchInput"
					type="text"
					:placeholder="searchPlaceholder"
					class="w-full bg-transparent outline-none text-neutral-900 dark:text-neutral-100 text-xs"
					@keydown.enter="handleSearch"
					@keydown.esc="handleClearSearch"
				/>

				<!-- Search mode toggle icon -->
				<button
					type="button"
					class="p-1 ml-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
					:class="contentSearch ? 'text-primary-600 dark:text-primary-400 font-bold' : 'text-neutral-400'"
					:title="contentSearch ? 'Mode: Search text inside files (tgrep)' : 'Mode: Search file names (tgrep)'"
					@click="toggleContentSearch"
				>
					<UIcon :name="contentSearch ? 'lucide:file-search' : 'lucide:file-text'" class="w-3.5 h-3.5" />
				</button>

				<!-- Indexing indicator badge -->
				<span
					v-if="isIndexing"
					class="flex items-center gap-1 text-[10px] text-amber-600 dark:text-amber-400 font-medium shrink-0 ml-1.5 px-1.5 py-0.5 rounded bg-amber-50 dark:bg-amber-950/50 border border-amber-200 dark:border-amber-800"
					title="tgrep is actively indexing this directory..."
				>
					<UIcon name="lucide:loader-2" class="w-3 h-3 animate-spin" />
					<span>Mengindeks…</span>
				</span>

				<!-- Clear search button -->
				<button
					v-if="searchInput"
					type="button"
					class="p-1 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
					title="Clear search (Esc)"
					@click="handleClearSearch"
				>
					<UIcon name="lucide:x" class="w-3.5 h-3.5" />
				</button>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	const {
		currentPath,
		isLoading,
		canGoBack,
		canGoForward,
		goBack,
		goForward,
		goUp,
		refresh,
		loadDirectory,
		triggerSearch,
		cancelSearch,
		contentSearch,
		getPathDisplayName
	} = useExplorer();

	const isEditingPath = ref(false);
	const inputPath = ref('');
	const pathInputRef = ref<HTMLInputElement | null>(null);
	const copied = ref(false);

	const searchInput = ref('');

	const searchPlaceholder = computed(() => {
		const folderName = getPathDisplayName(currentPath.value);
		return contentSearch.value ? `Search text in ${folderName} (tgrep)...` : `Search in ${folderName} (tgrep)...`;
	});

	// Split path into clickable segments
	const pathSegments = computed(() => {
		const p = currentPath.value;
		if (!p) return [{ name: 'This PC', path: '' }];

		const segments: { name: string; path: string }[] = [];
		const normalized = p.replace(/\//g, '\\');
		const parts = normalized.split('\\').filter(Boolean);

		let accumulated = '';
		for (let i = 0; i < parts.length; i++) {
			const part = parts[i];
			if (i === 0 && /^[a-zA-Z]:$/.test(part)) {
				accumulated = `${part}\\`;
			} else {
				accumulated = accumulated.endsWith('\\') ? `${accumulated}${part}` : `${accumulated}\\${part}`;
			}
			segments.push({
				name: part,
				path: accumulated
			});
		}

		return segments;
	});

	const enableEditMode = () => {
		inputPath.value = currentPath.value;
		isEditingPath.value = true;
		nextTick(() => {
			pathInputRef.value?.focus();
			pathInputRef.value?.select();
		});
	};

	const disableEditMode = () => {
		isEditingPath.value = false;
	};

	const submitPath = () => {
		if (inputPath.value.trim()) {
			loadDirectory(inputPath.value.trim());
		}
		isEditingPath.value = false;
	};

	const copyCurrentPath = async () => {
		try {
			await navigator.clipboard.writeText(currentPath.value);
			copied.value = true;
			setTimeout(() => {
				copied.value = false;
			}, 2000);
		} catch {
			// ignore
		}
	};

	const api = useExplorerApi();
	const isIndexing = ref(false);

	// Check tgrep indexing status periodically or when path changes
	const checkIndexingStatus = async () => {
		if (!currentPath.value) return;
		try {
			const status = await api.getSearchStatus(currentPath.value);
			isIndexing.value = status.toLowerCase().includes('indexing:') && !status.toLowerCase().includes('complete');
		} catch {
			isIndexing.value = false;
		}
	};

	watch(currentPath, () => {
		checkIndexingStatus();
	});

	// Debounced search on typing (200ms)
	let debounceTimer: any = null;
	watch(searchInput, (val) => {
		clearTimeout(debounceTimer);
		if (!val.trim()) {
			cancelSearch();
			return;
		}
		debounceTimer = setTimeout(() => {
			triggerSearch(val);
		}, 200);
	});

	const handleSearch = () => {
		if (searchInput.value.trim()) {
			triggerSearch(searchInput.value.trim());
		}
	};

	const handleClearSearch = () => {
		searchInput.value = '';
		cancelSearch();
	};

	const toggleContentSearch = () => {
		contentSearch.value = !contentSearch.value;
		if (searchInput.value.trim()) {
			triggerSearch(searchInput.value.trim());
		}
	};
</script>
