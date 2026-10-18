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
				v-if="isImageResizerOpen && imageResizerTarget"
				class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-8 select-none"
				tabindex="0"
				@keydown.esc="isImageResizerOpen = false"
			>
				<!-- Independent Backdrop Layer (no backdrop blur ghosting) -->
				<div class="absolute inset-0 bg-black/75" @click="isImageResizerOpen = false" />

				<div
					class="relative z-10 w-full max-w-2xl flex flex-col rounded-2xl border border-neutral-200 dark:border-neutral-700 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100"
					style="background-color: #ffffff; isolation: isolate;"
				>
					<!-- Header Bar -->
					<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-10 h-10 rounded-xl flex items-center justify-center bg-violet-500/10 text-violet-600 dark:text-violet-400">
								<UIcon name="lucide:image-down" class="w-6 h-6" />
							</div>
							<div class="min-w-0">
								<div class="flex items-center gap-2">
									<h2 class="text-base font-bold tracking-tight truncate">PowerToys Image Resizer</h2>
									<UBadge color="primary" variant="subtle" size="xs">Batch & Convert</UBadge>
								</div>
								<div class="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
									<span class="truncate max-w-sm">{{ imageResizerTarget.name }}</span>
									<span>•</span>
									<span class="font-mono">{{ formatBytes(imageResizerTarget.size) }}</span>
								</div>
							</div>
						</div>

						<button
							class="p-2 rounded-lg text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800 transition-colors"
							title="Close (Esc)"
							@click="isImageResizerOpen = false"
						>
							<UIcon name="lucide:x" class="w-5 h-5" />
						</button>
					</div>

					<!-- Body Content -->
					<div class="p-6 space-y-6 bg-white dark:bg-neutral-950 overflow-y-auto max-h-[75vh]">
						<!-- Dimension Presets -->
						<div class="space-y-3">
							<label class="text-xs font-bold uppercase tracking-wider text-neutral-500">Select Resolution Preset</label>
							<div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
								<button
									v-for="preset in presets"
									:key="preset.id"
									type="button"
									class="p-3 rounded-xl border text-left transition-all"
									:class="selectedPreset === preset.id
										? 'border-primary-500 bg-primary-50/50 dark:bg-primary-950/40 text-primary-900 dark:text-primary-100 ring-2 ring-primary-500/20'
										: 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-900 text-neutral-700 dark:text-neutral-300'"
									@click="applyPreset(preset)"
								>
									<div class="text-xs font-bold">{{ preset.name }}</div>
									<div class="text-[11px] font-mono text-neutral-500 mt-0.5">{{ preset.width }} × {{ preset.height }}</div>
								</button>
							</div>
						</div>

						<!-- Custom Dimension Inputs -->
						<div class="grid grid-cols-2 gap-4 p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/30">
							<div>
								<label class="text-xs font-semibold text-neutral-600 dark:text-neutral-400 block mb-1">Target Width (px)</label>
								<input
									v-model.number="targetWidth"
									type="number"
									min="16"
									max="8192"
									class="w-full px-3 py-1.5 text-xs font-mono rounded-lg border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 focus:outline-none focus:ring-2 focus:ring-primary-500"
									@input="selectedPreset = 'custom'"
								/>
							</div>
							<div>
								<label class="text-xs font-semibold text-neutral-600 dark:text-neutral-400 block mb-1">Target Height (px)</label>
								<input
									v-model.number="targetHeight"
									type="number"
									min="16"
									max="8192"
									class="w-full px-3 py-1.5 text-xs font-mono rounded-lg border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 focus:outline-none focus:ring-2 focus:ring-primary-500"
									@input="selectedPreset = 'custom'"
								/>
							</div>
						</div>

						<!-- Format Conversion & Quality -->
						<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
							<div class="space-y-1.5">
								<label class="text-xs font-bold uppercase tracking-wider text-neutral-500">Output Format</label>
								<div class="flex items-center gap-2">
									<button
										v-for="fmt in ['webp', 'jpeg', 'png']"
										:key="fmt"
										type="button"
										class="flex-1 py-2 px-3 rounded-lg border text-xs font-semibold uppercase transition-all"
										:class="targetFormat === fmt
											? 'border-primary-500 bg-primary-50 dark:bg-primary-950/60 text-primary-600 dark:text-primary-300 font-bold'
											: 'border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'"
										@click="targetFormat = fmt as any"
									>
										{{ fmt }}
									</button>
								</div>
							</div>

							<div class="space-y-1.5">
								<div class="flex items-center justify-between text-xs">
									<span class="font-bold uppercase tracking-wider text-neutral-500">Quality ({{ Math.round(quality * 100) }}%)</span>
									<span class="text-neutral-400 font-mono text-[11px]">{{ targetFormat === 'png' ? 'Lossless' : 'Compressed' }}</span>
								</div>
								<input
									v-model.number="quality"
									type="range"
									min="0.1"
									max="1"
									step="0.05"
									:disabled="targetFormat === 'png'"
									class="w-full accent-primary-500 cursor-pointer"
								/>
							</div>
						</div>

						<!-- Output Options -->
						<div class="space-y-2 pt-2 border-t border-neutral-100 dark:border-neutral-800">
							<label class="flex items-center gap-2.5 text-xs cursor-pointer text-neutral-700 dark:text-neutral-300">
								<input v-model="replaceOriginal" type="checkbox" class="rounded accent-primary-500" />
								<span>Replace original file (destructive)</span>
							</label>
							<div class="text-[11px] text-neutral-400 font-mono pl-6">
								Will output: {{ outputFilename }}
							</div>
						</div>
					</div>

					<!-- Footer Actions -->
					<div class="px-6 py-4 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex items-center justify-between">
						<UButton
							variant="outline"
							color="neutral"
							size="sm"
							@click="isImageResizerOpen = false"
						>
							Cancel
						</UButton>

						<UButton
							color="primary"
							size="sm"
							icon="lucide:check"
							:loading="isProcessing"
							@click="processResize"
						>
							Apply Resize & Convert
						</UButton>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script lang="ts" setup>
	const {
		isImageResizerOpen,
		imageResizerTarget,
		currentPath,
		refresh
	} = useExplorer();

	const api = useExplorerApi();
	const toast = useToast();

	const presets = [
		{ id: 'small', name: 'Small', width: 854, height: 480 },
		{ id: 'medium', name: 'Medium (HD)', width: 1366, height: 768 },
		{ id: 'large', name: 'Large (FHD)', width: 1920, height: 1080 },
		{ id: 'mobile', name: 'Mobile / Story', width: 1080, height: 1920 }
	];

	const selectedPreset = ref('medium');
	const targetWidth = ref(1366);
	const targetHeight = ref(768);
	const targetFormat = ref<'webp' | 'jpeg' | 'png'>('webp');
	const quality = ref(0.85);
	const replaceOriginal = ref(false);
	const isProcessing = ref(false);

	const outputFilename = computed(() => {
		if (!imageResizerTarget.value) return '';
		const base = imageResizerTarget.value.name.replace(/\.[^/.]+$/, '');
		const ext = targetFormat.value === 'jpeg' ? 'jpg' : targetFormat.value;
		if (replaceOriginal.value) {
			return `${base}.${ext}`;
		}
		return `${base}_${targetWidth.value}x${targetHeight.value}.${ext}`;
	});

	function applyPreset(p: typeof presets[0]) {
		selectedPreset.value = p.id;
		targetWidth.value = p.width;
		targetHeight.value = p.height;
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
	}

	async function processResize() {
		if (!imageResizerTarget.value) return;
		isProcessing.value = true;

		try {
			// Load the image as a data URL from the Rust backend
			const res = await api.getPreview(imageResizerTarget.value.path, true);
			if (!res.is_image || !res.preview) throw new Error('Not a readable image');

			const img = new Image();
			await new Promise<void>((resolve, reject) => {
				img.onload = () => resolve();
				img.onerror = () => reject(new Error('Failed to load image in canvas'));
				img.src = res.preview!;
			});

			// Draw and scale on canvas
			const canvas = document.createElement('canvas');
			canvas.width = targetWidth.value;
			canvas.height = targetHeight.value;
			const ctx = canvas.getContext('2d');
			if (!ctx) throw new Error('Canvas context unavailable');

			ctx.imageSmoothingEnabled = true;
			ctx.imageSmoothingQuality = 'high';

			// Maintain aspect ratio with fit/letterbox or stretch
			ctx.drawImage(img, 0, 0, targetWidth.value, targetHeight.value);

			const mimeType = targetFormat.value === 'jpeg' ? 'image/jpeg' : targetFormat.value === 'png' ? 'image/png' : 'image/webp';
			const dataUrl = canvas.toDataURL(mimeType, quality.value);
			const base64Content = dataUrl.split(',')[1];

			const destinationDir = currentPath.value;
			const fullNewPath = `${destinationDir}\\${outputFilename.value}`;

			// Save via createFile action
			await api.executeAction('createFile', destinationDir, {
				name: outputFilename.value,
				contentBase64: base64Content
			});

			toast.add({
				title: 'Image Resized Successfully',
				description: `Saved as ${outputFilename.value} (${targetWidth.value}×${targetHeight.value})`,
				color: 'success'
			});

			isImageResizerOpen.value = false;
			refresh();
		} catch (err: any) {
			toast.add({
				title: 'Resize Failed',
				description: err?.message || 'Error occurred while resizing image.',
				color: 'error'
			});
		} finally {
			isProcessing.value = false;
		}
	}
</script>
