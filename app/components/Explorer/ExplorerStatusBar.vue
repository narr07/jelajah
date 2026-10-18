<template>
	<footer class="h-6 px-3 bg-neutral-100/90 dark:bg-neutral-900/90 border-t border-neutral-200 dark:border-neutral-800 flex items-center justify-between text-[11px] text-neutral-500 dark:text-neutral-400 select-none shrink-0">
		<!-- Left info -->
		<div class="flex items-center gap-3">
			<span>{{ items.length }} items</span>

			<span v-if="selectedCount > 0" class="flex items-center gap-1.5 border-l border-neutral-300 dark:border-neutral-700 pl-3 text-neutral-800 dark:text-neutral-200 font-medium">
				<span>{{ selectedCount }} item{{ selectedCount > 1 ? 's' : '' }} selected</span>
				<span v-if="selectedSize > 0">({{ formatBytes(selectedSize) }})</span>
			</span>

			<!-- Git Status Badge -->
			<div
				v-if="gitStatus?.isRepo"
				class="flex items-center gap-1.5 border-l border-neutral-300 dark:border-neutral-700 pl-3 text-neutral-700 dark:text-neutral-300 cursor-pointer hover:text-primary-600 transition-colors"
				title="Git Repository Status"
				@click="handleGitClick"
			>
				<UIcon name="lucide:git-branch" class="w-3.5 h-3.5 text-primary-500" />
				<span class="font-semibold font-mono">{{ gitStatus.branch || 'HEAD' }}</span>

				<span v-if="(gitStatus.ahead ?? 0) > 0" class="text-emerald-600 font-mono text-[10px]" title="Commits ahead">
					↑{{ gitStatus.ahead }}
				</span>
				<span v-if="(gitStatus.behind ?? 0) > 0" class="text-amber-600 font-mono text-[10px]" title="Commits behind">
					↓{{ gitStatus.behind }}
				</span>

				<span
					v-if="modifiedCount > 0"
					class="px-1.5 py-0.2 rounded-full bg-amber-100 dark:bg-amber-950/60 text-amber-700 dark:text-amber-400 font-mono text-[10px] font-bold"
					title="Modified / Dirty files"
				>
					{{ modifiedCount }} modified
				</span>
			</div>
		</div>

		<!-- Right view toggles & advanced tools -->
		<div class="flex items-center gap-2">
			<!-- TreeSize Quick Access Button -->
			<button
				class="flex items-center gap-1 px-1.5 py-0.5 rounded text-neutral-600 dark:text-neutral-400 hover:text-emerald-600 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50 transition-colors"
				title="Open TreeSize Disk Space Analyzer"
				@click="openTreeSize()"
			>
				<UIcon name="lucide:pie-chart" class="w-3.5 h-3.5 text-emerald-500" />
				<span class="text-[10px] font-semibold">TreeSize</span>
			</button>

			<div class="h-3 w-px bg-neutral-300 dark:bg-neutral-700" />

			<!-- Dual Pane Toggle -->
			<button
				class="p-0.5 rounded transition-colors"
				:class="isDualPane ? 'bg-primary-100 dark:bg-primary-950/60 text-primary-700 dark:text-primary-300' : 'hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50'"
				title="Toggle Dual-Pane Split View"
				@click="toggleDualPane"
			>
				<UIcon name="lucide:columns-2" class="w-3.5 h-3.5" />
			</button>

			<button
				class="p-0.5 rounded transition-colors"
				:class="viewMode === 'details' ? 'bg-neutral-200 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100' : 'hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50'"
				title="Details view"
				@click="viewMode = 'details'"
			>
				<UIcon name="lucide:layout-list" class="w-3.5 h-3.5" />
			</button>

			<button
				class="p-0.5 rounded transition-colors"
				:class="viewMode === 'large' ? 'bg-neutral-200 dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100' : 'hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50'"
				title="Large icons view"
				@click="viewMode = 'large'"
			>
				<UIcon name="lucide:layout-grid" class="w-3.5 h-3.5" />
			</button>
		</div>
	</footer>
</template>

<script lang="ts" setup>
	const {
		items,
		selectedCount,
		selectedSize,
		viewMode,
		gitStatus,
		openTreeSize,
		isDualPane,
		toggleDualPane,
		currentPath
	} = useExplorer();

	const api = useExplorerApi();

	const modifiedCount = computed(() => {
		if (!gitStatus.value?.fileStatuses) return 0;
		return Object.keys(gitStatus.value.fileStatuses).length;
	});

	function handleGitClick() {
		// Open terminal in current git directory
		api.executeAction('openTerminal', currentPath.value);
	}

	function formatBytes(bytes: number) {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
	}
</script>
