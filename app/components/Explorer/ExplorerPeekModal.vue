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
				v-if="isPeekOpen && currentItem"
				class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-10 select-none"
				@keydown="handleKeyDown"
				tabindex="0"
				ref="modalRoot"
			>
				<!-- Independent Backdrop Layer -->
				<div class="absolute inset-0 bg-black/75" @click="isPeekOpen = false" />

				<div
					class="relative z-10 w-full max-w-5xl max-h-[88vh] flex flex-col rounded-2xl border border-neutral-200 dark:border-neutral-700 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100"
					style="background-color: #ffffff; isolation: isolate;"
				>
					<!-- Header Bar -->
					<div class="flex items-center justify-between px-5 py-3.5 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-8 h-8 rounded-lg flex items-center justify-center bg-primary-50 dark:bg-primary-950/50 text-primary-600 dark:text-primary-400">
								<UIcon :name="getFileIcon(currentItem.name, currentItem.is_dir)" class="w-5 h-5 shrink-0" />
							</div>
							<div class="min-w-0">
								<h3 class="text-sm font-semibold truncate leading-tight">{{ currentItem.name }}</h3>
								<div class="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
									<span>{{ currentItem.is_dir ? 'Folder' : formatBytes(currentItem.size) }}</span>
									<span>•</span>
									<span>{{ formatDate(currentItem.modified) }}</span>
									<span v-if="currentItem.extension" class="uppercase font-mono text-[10px] px-1.5 py-0.2 rounded bg-neutral-200/60 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300">
										{{ currentItem.extension }}
									</span>
								</div>
							</div>
						</div>

						<!-- Action Controls -->
						<div class="flex items-center gap-1.5 shrink-0">
							<UButton
								variant="ghost"
								size="sm"
								icon="lucide:copy"
								title="Copy Path"
								@click="copyPath"
							/>
							<UButton
								variant="ghost"
								size="sm"
								icon="lucide:external-link"
								title="Open with Default App"
								@click="openItem(currentItem)"
							>
								Open
							</UButton>
							<div class="w-[1px] h-4 bg-neutral-200 dark:bg-neutral-700 mx-1"></div>
							<UButton
								variant="ghost"
								size="sm"
								icon="lucide:x"
								title="Close (Esc)"
								@click="isPeekOpen = false"
							/>
						</div>
					</div>

					<!-- Content Preview Area -->
					<div class="flex-1 overflow-auto p-4 sm:p-6 min-h-[320px] max-h-[68vh] flex items-center justify-center relative" style="background-color: #f8fafc;">
						<!-- Loading indicator -->
						<div v-if="previewLoading" class="flex flex-col items-center gap-3 text-neutral-400">
							<UIcon name="lucide:loader-2" class="w-7 h-7 animate-spin text-primary-500" />
							<span class="text-xs">Loading preview...</span>
						</div>

						<!-- 1. Image Preview -->
						<div v-else-if="activePreview?.media === 'image'" class="flex flex-col items-center justify-center max-w-full max-h-full">
							<div class="p-2 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-checkered shadow-inner overflow-hidden max-w-full max-h-[58vh]">
								<img
									:src="fileSrc(currentItem.path)"
									:alt="currentItem.name"
									class="max-w-full max-h-[55vh] object-contain rounded-lg mx-auto"
								/>
							</div>
							<div class="mt-2 text-xs text-neutral-500 dark:text-neutral-400 font-mono">
								{{ formatBytes(currentItem.size) }}
							</div>
						</div>

						<!-- PDF / video / audio: streamed by the webview (File Explorer add-ons) -->
						<iframe
							v-else-if="activePreview?.media === 'pdf'"
							:src="fileSrc(currentItem.path)"
							class="w-full h-[60vh] rounded-xl border border-neutral-200 dark:border-neutral-800"
						/>
						<video
							v-else-if="activePreview?.media === 'video'"
							:src="fileSrc(currentItem.path)"
							controls
							class="max-w-full max-h-[58vh] rounded-xl"
						/>
						<audio
							v-else-if="activePreview?.media === 'audio'"
							:src="fileSrc(currentItem.path)"
							controls
							class="w-full max-w-md"
						/>

						<!-- 2. Text / Source Code Preview -->
						<div v-else-if="activePreview && !activePreview.is_binary && activePreview.preview" class="w-full h-full flex flex-col">
							<div class="flex items-center justify-between mb-2 px-1 text-xs text-neutral-500 dark:text-neutral-400">
								<span>Lines: {{ lineCount }} • Chars: {{ activePreview.preview.length }}</span>
								<div class="flex items-center gap-2">
									<UButton
										v-if="currentItem.extension === 'md'"
										variant="soft"
										size="xs"
										:icon="showRenderedMarkdown ? 'lucide:code' : 'lucide:book-open'"
										@click="showRenderedMarkdown = !showRenderedMarkdown"
									>
										{{ showRenderedMarkdown ? 'View Source' : 'Preview Markdown' }}
									</UButton>
									<UButton
										variant="ghost"
										size="xs"
										icon="lucide:clipboard"
										@click="copyTextContent"
									>
										{{ copiedText ? 'Copied!' : 'Copy Code' }}
									</UButton>
								</div>
							</div>

							<!-- Rendered Markdown -->
							<div
								v-if="currentItem.extension === 'md' && showRenderedMarkdown"
								class="flex-1 w-full overflow-auto p-5 rounded-xl bg-white border border-neutral-200 dark:border-neutral-800 prose dark:prose-invert max-w-none text-xs leading-relaxed"
								style="background-color: #ffffff;"
							>
								<div class="whitespace-pre-wrap font-sans">{{ activePreview.preview }}</div>
							</div>

							<!-- Code View with Line Numbers -->
							<div
								v-else
								class="flex-1 w-full overflow-auto p-4 rounded-xl text-neutral-100 font-mono text-xs leading-relaxed border border-neutral-800 shadow-inner flex gap-3"
								style="background-color: #18181b;"
							>
								<div class="select-none text-neutral-500 text-right pr-3 border-r border-neutral-800 space-y-0.5">
									<div v-for="n in lineCount" :key="n">{{ n }}</div>
								</div>
								<div class="flex-1 whitespace-pre overflow-x-auto text-neutral-200 space-y-0.5">
									{{ activePreview.preview }}
								</div>
							</div>
						</div>

						<!-- 3. Office & co: Windows draws the file's preview handler over this box -->
						<div v-else-if="useNative" ref="nativeHost" class="w-full h-[60vh] rounded-xl bg-white">
							<div v-if="!nativeShown" class="w-full h-full flex items-center justify-center text-neutral-400">
								<UIcon name="lucide:loader-2" class="w-7 h-7 animate-spin" />
							</div>
						</div>

						<!-- 4. Everything else, Quick Look style: Explorer's thumbnail (first page of
						     a document, a video frame, folder contents, or the 256 px icon) -->
						<div v-else-if="thumbnail" class="flex flex-col items-center gap-3 max-w-full">
							<img :src="thumbnail" :alt="currentItem.name" class="max-w-full max-h-[56vh] object-contain drop-shadow-xl" draggable="false">
							<p class="text-xs text-neutral-500 dark:text-neutral-400 font-mono break-all text-center max-w-lg">{{ currentItem.path }}</p>
						</div>

						<!-- 5. Folder Summary (while its thumbnail loads) -->
						<div v-else-if="currentItem.is_dir" class="text-center p-8 max-w-md">
							<div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center bg-amber-50 dark:bg-amber-950/40 text-amber-500 shadow-sm border border-amber-200/50 dark:border-amber-900/50">
								<UIcon name="lucide:folder" class="w-8 h-8" />
							</div>
							<h4 class="text-base font-semibold">{{ currentItem.name }}</h4>
							<p class="text-xs text-neutral-500 dark:text-neutral-400 mt-1 font-mono break-all">{{ currentItem.path }}</p>
							<div class="mt-5 flex justify-center gap-2">
								<UButton
									color="primary"
									icon="lucide:folder-open"
									@click="openFolder(currentItem)"
								>
									Enter Folder
								</UButton>
							</div>
						</div>

						<!-- 4. Binary / Unsupported File -->
						<div v-else class="text-center p-8 max-w-md">
							<div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center bg-neutral-100 dark:bg-neutral-800 text-neutral-500 shadow-sm border border-neutral-200 dark:border-neutral-700">
								<UIcon :name="getFileIcon(currentItem.name, false)" class="w-8 h-8" />
							</div>
							<h4 class="text-base font-semibold">{{ currentItem.name }}</h4>
							<p class="text-xs text-neutral-500 dark:text-neutral-400 mt-1">
								{{ formatBytes(currentItem.size) }} • {{ currentItem.extension.toUpperCase() || 'Binary' }} File
							</p>
							<div class="mt-5 flex justify-center gap-2">
								<UButton
									color="primary"
									icon="lucide:external-link"
									@click="openItem(currentItem)"
								>
									Open in Default App
								</UButton>
							</div>
						</div>
					</div>

					<!-- Bottom Navigation Hints Bar -->
					<div class="px-5 py-2.5 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400 font-medium">
						<div class="flex items-center gap-3">
							<span class="flex items-center gap-1">
								<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono text-[11px]">↑</kbd>
								<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono text-[11px]">↓</kbd>
								Next/Previous item
							</span>
							<span>•</span>
							<span class="flex items-center gap-1">
								<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono text-[11px]">Enter</kbd>
								Open
							</span>
						</div>

						<div class="flex items-center gap-1">
							<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono text-[11px]">Space</kbd>
							<kbd class="px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 font-mono text-[11px]">Esc</kbd>
							Close Peek
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script lang="ts" setup>
	import type { FileItem } from '~/composables/useExplorerApi';
	import { fileSrc } from '~/composables/useExplorerApi';

	const {
		isPeekOpen,
		selectedItem,
		activePreview,
		previewLoading,
		navigateSelection,
		openItem,
		loadDirectory
	} = useExplorer();

	const api = useExplorerApi();
	const modalRoot = ref<HTMLElement | null>(null);
	const showRenderedMarkdown = ref(true);
	const copiedText = ref(false);

	const currentItem = computed(() => selectedItem.value);

	// Quick Look for everything else, in order:
	// 1. the type's Windows preview handler (Office, ...), drawn natively over `nativeHost`
	// 2. Explorer's thumbnail (first page, video frame, folder contents, big icon)
	const thumbnail = ref<string | null>(null);
	const nativeHost = ref<HTMLElement | null>(null);
	const useNative = ref(false); // render the host box
	const nativeShown = ref(false); // a handler is drawing in it
	let resizeObs: ResizeObserver | null = null;

	const rectOf = (el: HTMLElement) => {
		const r = el.getBoundingClientRect();
		return { x: r.left, y: r.top, w: r.width, h: r.height };
	};
	const syncNative = () => {
		if (nativeShown.value && nativeHost.value) api.previewMove(rectOf(nativeHost.value));
	};
	const closeNative = () => {
		if (useNative.value || nativeShown.value) api.previewClose();
		useNative.value = false;
		nativeShown.value = false;
		resizeObs?.disconnect();
		resizeObs = null;
	};

	watch([currentItem, activePreview, previewLoading, isPeekOpen], async () => {
		thumbnail.value = null;
		closeNative();
		const item = currentItem.value;
		const p = activePreview.value;
		if (!isPeekOpen.value || !item || previewLoading.value) return;
		if (!item.is_dir && (p?.media || (p && !p.is_binary && p.preview))) return;
		const stale = () => !isPeekOpen.value || currentItem.value?.path !== item.path;

		if (!item.is_dir) {
			useNative.value = true;
			await nextTick();
			await new Promise(r => setTimeout(r, 180)); // let the open animation finish: the rect must be final
			if (stale() || !nativeHost.value) return;
			const ok = await api.previewOpen(item.path, rectOf(nativeHost.value)).catch(() => false);
			if (stale()) return;
			if (ok) {
				nativeShown.value = true;
				resizeObs = new ResizeObserver(syncNative);
				resizeObs.observe(nativeHost.value);
				return;
			}
			useNative.value = false; // no handler for this type
		}

		const size = Math.min(1024, Math.round(480 * (window.devicePixelRatio || 1)));
		const url = await api.fileThumbnail(item.path, size).catch(() => null);
		if (!stale()) thumbnail.value = url;
	});
	onMounted(() => window.addEventListener('resize', syncNative));
	onBeforeUnmount(() => {
		window.removeEventListener('resize', syncNative);
		closeNative();
	});

	const lineCount = computed(() => {
		if (activePreview.value && !activePreview.value.is_binary && activePreview.value.preview) {
			return activePreview.value.preview.split('\n').length;
		}
		return 0;
	});

	watch(isPeekOpen, (open) => {
		if (open) {
			nextTick(() => {
				modalRoot.value?.focus();
			});
		}
	});

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === 'Escape') { // Space is handled globally in pages/index.vue (toggle)
			e.preventDefault();
			isPeekOpen.value = false;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			navigateSelection('up');
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			navigateSelection('down');
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (currentItem.value) {
				if (currentItem.value.is_dir) {
					openFolder(currentItem.value);
				} else {
					openItem(currentItem.value);
				}
			}
		}
	};

	const openFolder = (item: FileItem) => {
		isPeekOpen.value = false;
		loadDirectory(item.path);
	};

	const copyPath = () => {
		if (currentItem.value && typeof navigator !== 'undefined') {
			navigator.clipboard.writeText(currentItem.value.path);
		}
	};

	const copyTextContent = () => {
		if (activePreview.value?.preview && typeof navigator !== 'undefined') {
			navigator.clipboard.writeText(activePreview.value.preview);
			copiedText.value = true;
			setTimeout(() => {
				copiedText.value = false;
			}, 2000);
		}
	};

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
				return 'lucide:video';
			case 'mp3':
			case 'wav':
			case 'ogg':
				return 'lucide:music';
			case 'zip':
			case 'rar':
			case '7z':
				return 'lucide:archive';
			case 'ts':
			case 'js':
			case 'vue':
			case 'rs':
			case 'py':
			case 'html':
			case 'css':
			case 'json':
			case 'md':
				return 'lucide:file-code-2';
			case 'exe':
			case 'bat':
			case 'cmd':
				return 'lucide:terminal';
			case 'pdf':
				return 'lucide:file-text';
			default:
				return 'lucide:file';
		}
	};

	const formatBytes = (bytes: number) => {
		if (!bytes) return '0 B';
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

<style scoped>
.bg-checkered {
	background-color: #262626;
	background-image:
		linear-gradient(45deg, #1c1c1c 25%, transparent 25%),
		linear-gradient(-45deg, #1c1c1c 25%, transparent 25%),
		linear-gradient(45deg, transparent 75%, #1c1c1c 75%),
		linear-gradient(-45deg, transparent 75%, #1c1c1c 75%);
	background-size: 16px 16px;
	background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
}
</style>
