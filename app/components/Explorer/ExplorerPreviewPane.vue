<template>
	<aside
		v-if="showPreviewPane"
		class="w-72 shrink-0 h-full overflow-y-auto bg-neutral-50/70 dark:bg-neutral-900/70 border-l border-neutral-200 dark:border-neutral-800 p-3 text-xs select-none flex flex-col"
	>
		<!-- Header -->
		<div class="flex items-center justify-between pb-2 mb-2 border-b border-neutral-200 dark:border-neutral-800">
			<span class="font-semibold text-neutral-900 dark:text-neutral-100">Details & Preview</span>
			<button
				class="p-1 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/50"
				@click="showPreviewPane = false"
			>
				<UIcon name="lucide:x" class="w-3.5 h-3.5" />
			</button>
		</div>

		<!-- Selected Item Details -->
		<div v-if="selectedCount === 1" class="flex-1 flex flex-col">
			<!-- Preview box -->
			<div class="flex flex-col items-center justify-center p-3 mb-3 rounded-lg bg-white dark:bg-neutral-800/60 border border-neutral-200/80 dark:border-neutral-700/60 min-h-[140px] max-h-[220px] overflow-hidden">
				<!-- Image Preview -->
				<img
					v-if="activePreview?.media === 'image' && selectedItem"
					:src="fileSrc(selectedItem.path)"
					loading="lazy"
					class="max-h-36 max-w-full object-contain rounded"
				/>
				<video
					v-else-if="activePreview?.media === 'video' && selectedItem"
					:src="fileSrc(selectedItem.path)"
					controls
					preload="metadata"
					class="max-h-48 max-w-full rounded"
				/>
				<audio
					v-else-if="activePreview?.media === 'audio' && selectedItem"
					:src="fileSrc(selectedItem.path)"
					controls
					preload="metadata"
					class="w-full"
				/>
				<iframe
					v-else-if="activePreview?.media === 'pdf' && selectedItem"
					:src="fileSrc(selectedItem.path)"
					class="w-full h-52 rounded"
				/>

				<!-- Text / Code Preview snippet -->
				<div
					v-else-if="!activePreview?.is_dir && activePreview?.preview"
					class="w-full h-full font-mono text-[10px] text-neutral-700 dark:text-neutral-300 overflow-auto whitespace-pre leading-relaxed select-text p-1"
				>
					{{ activePreview.preview }}
				</div>

				<!-- Generic Icon fallback -->
				<div v-else class="flex flex-col items-center">
					<UIcon
						:name="selectedItem?.is_dir ? 'lucide:folder' : 'lucide:file'"
						class="w-16 h-16"
						:class="selectedItem?.is_dir ? 'text-amber-500' : 'text-neutral-400'"
					/>
				</div>
			</div>

			<!-- Metadata details -->
			<div class="space-y-2 text-[11px]">
				<div>
					<div class="text-neutral-400 uppercase text-[10px] font-semibold">Name</div>
					<div class="font-medium text-neutral-900 dark:text-neutral-100 truncate select-text">
						{{ selectedItem?.name }}
					</div>
				</div>

				<div v-if="!selectedItem?.is_dir">
					<div class="text-neutral-400 uppercase text-[10px] font-semibold">Size</div>
					<div class="text-neutral-700 dark:text-neutral-300">
						{{ formatBytes(selectedItem?.size || 0) }} ({{ selectedItem?.size.toLocaleString() }} bytes)
					</div>
				</div>

				<div>
					<div class="text-neutral-400 uppercase text-[10px] font-semibold">Date modified</div>
					<div class="text-neutral-700 dark:text-neutral-300">
						{{ formatDate(selectedItem?.modified || 0) }}
					</div>
				</div>

				<div>
					<div class="text-neutral-400 uppercase text-[10px] font-semibold">Path</div>
					<div class="text-neutral-600 dark:text-neutral-400 font-mono text-[10px] break-all select-text">
						{{ selectedItem?.path }}
					</div>
				</div>
			</div>
		</div>

		<!-- Multiple items or None selected -->
		<div v-else-if="selectedCount > 1" class="flex-1 flex flex-col items-center justify-center text-center text-neutral-400">
			<UIcon name="lucide:files" class="w-12 h-12 stroke-[1.2] mb-2 text-neutral-300 dark:text-neutral-700" />
			<span class="font-medium text-neutral-600 dark:text-neutral-300">{{ selectedCount }} items selected</span>
			<span class="text-[11px] text-neutral-400 mt-1">Total size: {{ formatBytes(selectedSize) }}</span>
		</div>

		<div v-else class="flex-1 flex flex-col items-center justify-center text-center text-neutral-400">
			<UIcon name="lucide:mouse-pointer-click" class="w-10 h-10 stroke-[1.2] mb-2 text-neutral-300 dark:text-neutral-700" />
			<span class="text-neutral-500">Select an item to view its details and preview</span>
		</div>
	</aside>
</template>

<script lang="ts" setup>
	import { fileSrc } from '~/composables/useExplorerApi';

	const {
		showPreviewPane,
		selectedPaths,
		selectedCount,
		selectedSize,
		items,
		activePreview
	} = useExplorer();

	const selectedItem = computed(() => {
		if (selectedCount.value !== 1) return null;
		const path = Array.from(selectedPaths.value)[0];
		return items.value.find(i => i.path === path) || null;
	});

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
		return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`;
	};
</script>
