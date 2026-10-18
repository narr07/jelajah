<template>
	<div
		class="fixed inset-0 z-50"
		@click.stop="$emit('close')"
		@contextmenu.prevent.stop="$emit('close')"
	>
		<div
			class="absolute min-w-56 p-1.5 rounded-xl bg-white/95 dark:bg-neutral-800/95 backdrop-blur-md border border-neutral-200/80 dark:border-neutral-700/80 shadow-xl text-xs text-neutral-800 dark:text-neutral-200 select-none animate-in fade-in zoom-in-95 duration-100"
			:style="{ top: `${adjustedY}px`, left: `${adjustedX}px` }"
			@click.stop
		>
			<!-- Top Action Row (Windows 11 Fluent Style) -->
			<div v-if="targetItem" class="flex items-center justify-around pb-1.5 mb-1.5 border-b border-neutral-100 dark:border-neutral-700/60">
				<button
					class="p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 transition-colors"
					title="Cut (Ctrl+X)"
					@click="handleCut"
				>
					<UIcon name="lucide:scissors" class="w-4 h-4" />
				</button>
				<button
					class="p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 transition-colors"
					title="Copy (Ctrl+C)"
					@click="handleCopy"
				>
					<UIcon name="lucide:copy" class="w-4 h-4" />
				</button>
				<button
					class="p-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 transition-colors"
					title="Rename (F2)"
					@click="handleRename"
				>
					<UIcon name="lucide:pen-line" class="w-4 h-4" />
				</button>
				<button
					class="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-950/40 text-red-600 dark:text-red-400 transition-colors"
					title="Delete (Del)"
					@click="handleDelete"
				>
					<UIcon name="lucide:trash-2" class="w-4 h-4" />
				</button>
			</div>

			<!-- Main Menu Items -->
			<div class="space-y-0.5">
				<button
					v-if="targetItem"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors font-medium"
					@click="handleOpen"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:external-link" class="w-4 h-4 text-neutral-500" />
						<span>Open</span>
					</div>
					<span class="text-[10px] text-neutral-400">Enter</span>
				</button>

				<button
					v-if="targetItem"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors font-medium text-primary-600 dark:text-primary-400"
					@click="handlePeek"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:eye" class="w-4 h-4 text-primary-500" />
						<span>Quick Peek</span>
					</div>
					<span class="text-[10px] text-neutral-400">Space</span>
				</button>

				<button
					v-if="targetItem"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handlePowerRename"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:spell-check" class="w-4 h-4 text-blue-500" />
						<span>PowerRename</span>
					</div>
					<span class="text-[10px] text-neutral-400">Ctrl+M</span>
				</button>

				<button
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handlePasteAsFile"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:clipboard-paste" class="w-4 h-4 text-neutral-500" />
						<span>Paste as File</span>
					</div>
					<span class="text-[10px] text-neutral-400">Ctrl+Shift+V</span>
				</button>

				<div class="my-1 border-t border-neutral-100 dark:border-neutral-700/60" />

				<button
					v-if="targetItem?.is_dir"
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleOpenInNewTab"
				>
					<UIcon name="lucide:folder-plus" class="w-4 h-4 text-neutral-500" />
					<span>Open in new tab</span>
				</button>

				<button
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleOpenTerminal"
				>
					<UIcon name="lucide:terminal" class="w-4 h-4 text-neutral-500" />
					<span>Open in Terminal</span>
				</button>

				<button
					v-if="targetItem?.is_dir"
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleIndexTgrep"
				>
					<UIcon name="lucide:zap" class="w-4 h-4 text-amber-500" />
					<span>Index with tgrep</span>
				</button>
				<button
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleTreeSize"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:pie-chart" class="w-4 h-4 text-emerald-500" />
						<span>TreeSize Disk Space</span>
					</div>
					<span class="text-[10px] text-neutral-400">JAM Style</span>
				</button>

				<button
					v-if="isImageFile"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors text-violet-600 dark:text-violet-400 font-medium"
					@click="handleImageResize"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:image-down" class="w-4 h-4 text-violet-500" />
						<span>Resize & Convert Image</span>
					</div>
					<span class="text-[10px] text-neutral-400">PowerToys</span>
				</button>

				<button
					v-if="isImageFile"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleOcr"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:scan-text" class="w-4 h-4 text-sky-500" />
						<span>Copy Text from Image</span>
					</div>
					<span class="text-[10px] text-neutral-400">Text Extractor</span>
				</button>

				<button
					v-if="targetItem && !targetItem.is_dir"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleHash"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:fingerprint" class="w-4 h-4 text-blue-500" />
						<span>Checksum & Hashes</span>
					</div>
					<span class="text-[10px] text-neutral-400">Verify</span>
				</button>

				<div class="my-1 border-t border-neutral-100 dark:border-neutral-700/60" />

				<button
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleCopyPath"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:link-2" class="w-4 h-4 text-neutral-500" />
						<span>Copy as path</span>
					</div>
					<span class="text-[10px] text-neutral-400">Ctrl+Shift+C</span>
				</button>

				<button
					v-if="targetItem"
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleProperties"
				>
					<UIcon name="lucide:info" class="w-4 h-4 text-neutral-500" />
					<span>Properties</span>
				</button>

				<div v-if="targetItem" class="my-1 border-t border-neutral-100 dark:border-neutral-700/60" />
				<button
					v-if="targetItem"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg hover:bg-neutral-100 dark:hover:bg-neutral-700 text-left transition-colors"
					@click="handleMoreOptions"
				>
					<div class="flex items-center gap-2.5">
						<UIcon name="lucide:square-arrow-out-up-right" class="w-4 h-4 text-neutral-500" />
						<span>Show more options</span>
					</div>
					<span class="text-[10px] text-neutral-400">Shift+F10</span>
				</button>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
	import type { FileItem } from '~/composables/useExplorerApi';

	const props = defineProps<{
		x: number;
		y: number;
		targetItem: FileItem | null;
	}>();

	const emit = defineEmits<{
		(e: 'close'): void;
	}>();

	const {
		currentPath,
		clipboard,
		addTab,
		openItem,
		refresh,
		togglePeek,
		openPowerRename,
		pasteAsFile,
		selectItem,
		openTreeSize,
		openHashModal,
		openImageResizer,
		deletePaths
	} = useExplorer();
	const api = useExplorerApi();
	const toast = useToast();

	const isImageFile = computed(() => {
		if (!props.targetItem || props.targetItem.is_dir) return false;
		const ext = props.targetItem.extension.toLowerCase();
		return ['png', 'jpg', 'jpeg', 'webp', 'gif', 'bmp', 'svg', 'avif'].includes(ext);
	});

	const handleTreeSize = () => {
		const target = props.targetItem?.is_dir ? props.targetItem.path : currentPath.value;
		openTreeSize(target);
		emit('close');
	};

	const handleImageResize = () => {
		if (props.targetItem) {
			openImageResizer(props.targetItem);
		}
		emit('close');
	};

	const handleHash = () => {
		if (props.targetItem) {
			openHashModal(props.targetItem);
		}
		emit('close');
	};

	// Keep menu inside viewport boundaries
	const adjustedX = computed(() => {
		const menuWidth = 240;
		if (typeof window !== 'undefined' && props.x + menuWidth > window.innerWidth) {
			return props.x - menuWidth;
		}
		return props.x;
	});

	const adjustedY = computed(() => {
		const menuHeight = 260;
		if (typeof window !== 'undefined' && props.y + menuHeight > window.innerHeight) {
			return props.y - menuHeight;
		}
		return props.y;
	});

	const handlePeek = () => {
		if (props.targetItem) {
			selectItem(props.targetItem.path, false);
			togglePeek();
		}
		emit('close');
	};

	const handlePowerRename = () => {
		if (props.targetItem) {
			selectItem(props.targetItem.path, false);
		}
		openPowerRename();
		emit('close');
	};

	const handlePasteAsFile = async () => {
		emit('close');
		await pasteAsFile();
	};

	const handleOpen = () => {
		if (props.targetItem) {
			openItem(props.targetItem);
		}
		emit('close');
	};

	const handleOpenInNewTab = () => {
		if (props.targetItem?.is_dir) {
			addTab(props.targetItem.path);
		}
		emit('close');
	};

	const handleOpenTerminal = () => {
		const target = props.targetItem?.is_dir ? props.targetItem.path : currentPath.value;
		api.executeAction('openTerminal', target);
		emit('close');
	};

	const handleIndexTgrep = async () => {
		const target = props.targetItem?.is_dir ? props.targetItem.path : currentPath.value;
		emit('close');
		try {
			const res = await api.buildTgrepIndex(target);
			toast.add({ title: 'tgrep', description: res.message, color: 'success' });
		} catch (e: any) {
			toast.add({ title: 'Indexing failed', description: e.message, color: 'error' });
		}
	};

	const handleCut = () => {
		if (props.targetItem) {
			clipboard.value = {
				paths: [props.targetItem.path],
				action: 'cut'
			};
		}
		emit('close');
	};

	const handleCopy = () => {
		if (props.targetItem) {
			clipboard.value = {
				paths: [props.targetItem.path],
				action: 'copy'
			};
		}
		emit('close');
	};

	const handleRename = () => {
		emit('close');
		if (!props.targetItem) return;
		const newName = prompt('Enter new name:', props.targetItem.name);
		if (newName && newName !== props.targetItem.name) {
			const parent = props.targetItem.path.replace(/[\\/][^\\/]+$/, '');
			const newFullPath = `${parent}\\${newName}`;
			api.executeAction('rename', props.targetItem.path, { newPath: newFullPath })
				.then(() => refresh())
				.catch(err => toast.add({ title: 'Error', description: err.message, color: 'error' }));
		}
	};

	const handleDelete = async () => {
		emit('close');
		if (!props.targetItem) return;
		try {
			if (await deletePaths([props.targetItem.path])) {
				toast.add({ title: 'Moved to Recycle Bin' });
			}
		} catch (e: any) {
			toast.add({ title: 'Error', description: e.message, color: 'error' });
		}
	};

	const handleCopyPath = async () => {
		const p = props.targetItem ? props.targetItem.path : currentPath.value;
		try {
			await navigator.clipboard.writeText(p);
			toast.add({ title: 'Copied', description: 'Path copied to clipboard' });
		} catch {
			// ignore
		}
		emit('close');
	};

	const handleProperties = () => {
		emit('close');
		if (props.targetItem) {
			api.showProperties(props.targetItem.path).catch((e: Error) => toast.add({ title: 'Error', description: e.message, color: 'error' }));
		}
	};

	// Windows 11 "Show more options": Explorer's own context menu with every shell extension.
	const handleMoreOptions = () => {
		emit('close');
		if (props.targetItem) {
			api.shellContextMenu(props.targetItem.path).then(() => refresh()).catch((e: Error) => toast.add({ title: 'Error', description: e.message, color: 'error' }));
		}
	};

	// PowerToys Text Extractor, file-based: OCR the image and put the text on the clipboard.
	const handleOcr = async () => {
		emit('close');
		if (!props.targetItem) return;
		try {
			const text = await api.ocrImage(props.targetItem.path);
			await navigator.clipboard.writeText(text);
			toast.add({ title: text ? 'Text copied' : 'No text found', description: text.slice(0, 120), color: text ? 'success' : 'neutral' });
		} catch (e: any) {
			toast.add({ title: 'Text Extractor', description: e.message, color: 'error' });
		}
	};
</script>
