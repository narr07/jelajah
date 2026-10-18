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
				v-if="isTreeSizeOpen"
				class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-8 select-none"
				tabindex="0"
				@keydown.esc="isTreeSizeOpen = false"
			>
				<!-- Independent Backdrop Layer (no backdrop-filter blur ghosting) -->
				<div class="absolute inset-0 bg-black/75" @click="isTreeSizeOpen = false" />

				<div
					class="relative z-10 w-full max-w-6xl max-h-[90vh] flex flex-col rounded-2xl border border-neutral-200 dark:border-neutral-700 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100"
					style="background-color: #ffffff; isolation: isolate;"
				>
					<!-- Header Bar -->
					<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-xl flex items-center justify-center bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
								<UIcon name="lucide:pie-chart" class="w-6 h-6" />
							</div>
							<div>
								<div class="flex items-center gap-2">
									<h2 class="text-base font-bold tracking-tight">TreeSize Disk Space Analyzer</h2>
									<UBadge color="success" variant="subtle" size="xs">JAM Software Style</UBadge>
								</div>
								<p class="text-xs text-neutral-500 dark:text-neutral-400 font-mono truncate max-w-xl mt-0.5">
									{{ treeSizeData?.targetPath || currentPath }}
								</p>
							</div>
						</div>

						<div class="flex items-center gap-2">
							<UButton
								variant="ghost"
								color="neutral"
								size="sm"
								icon="lucide:arrow-up"
								:disabled="treeSizeLoading || !treeSizeData"
								title="Parent folder"
								@click="openTreeSize(getFolderOf(treeSizeData!.targetPath.replace(/[\\/]$/, '')))"
							>
								Up
							</UButton>
							<UButton
								variant="ghost"
								color="neutral"
								size="sm"
								icon="lucide:refresh-cw"
								:loading="treeSizeLoading"
								title="Rescan Directory"
								@click="openTreeSize(treeSizeData?.targetPath, true)"
							>
								Rescan
							</UButton>
							<button
								class="p-2 rounded-lg text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800 transition-colors"
								title="Close (Esc)"
								@click="isTreeSizeOpen = false"
							>
								<UIcon name="lucide:x" class="w-5 h-5" />
							</button>
						</div>
					</div>

					<!-- Content Body -->
					<div class="flex-1 overflow-y-auto p-6 space-y-6 bg-white dark:bg-neutral-950">
						<!-- Loading Skeleton -->
						<div v-if="treeSizeLoading" class="py-16 flex flex-col items-center justify-center space-y-4">
							<UIcon name="lucide:loader-2" class="w-10 h-10 text-primary-500 animate-spin" />
							<div class="text-center">
								<div class="text-sm font-semibold">Scanning directory trees & file allocations...</div>
								<div class="text-xs text-neutral-400 mt-1 font-mono">{{ treeSizeProgress.toLocaleString() }} files scanned</div>
							</div>
						</div>

						<div v-else-if="treeSizeData" class="space-y-6">
							<!-- Summary Cards -->
							<div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
								<div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/70 dark:bg-neutral-900/50">
									<div class="text-xs text-neutral-500 font-medium">Total Allocated Space</div>
									<div class="text-2xl font-black text-emerald-600 dark:text-emerald-400 mt-1">
										{{ formatBytes(treeSizeData.totalSize) }}
									</div>
									<div class="text-[11px] text-neutral-400 mt-0.5">Measured across immediate branches</div>
								</div>

								<div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/70 dark:bg-neutral-900/50">
									<div class="text-xs text-neutral-500 font-medium">Total Files Count</div>
									<div class="text-2xl font-black text-blue-600 dark:text-blue-400 mt-1">
										{{ treeSizeData.totalFiles.toLocaleString() }}
									</div>
									<div class="text-[11px] text-neutral-400 mt-0.5">Scanned files in directory tree</div>
								</div>

								<div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/70 dark:bg-neutral-900/50">
									<div class="text-xs text-neutral-500 font-medium">Total Folders Count</div>
									<div class="text-2xl font-black text-purple-600 dark:text-purple-400 mt-1">
										{{ treeSizeData.totalFolders.toLocaleString() }}
									</div>
									<div class="text-[11px] text-neutral-400 mt-0.5">Subdirectories cataloged</div>
								</div>
							</div>

							<!-- Nuxt UI Chart & Categories Breakdown -->
							<div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
								<!-- Nuxt UI Donut Chart -->
								<div class="lg:col-span-5 p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/30 flex flex-col">
									<div class="flex items-center justify-between mb-4">
										<h3 class="text-xs font-bold uppercase tracking-wider text-neutral-500">Storage by File Type</h3>
										<span class="text-xs text-neutral-400 font-mono">{{ chartData.length }} categories</span>
									</div>

									<div class="flex-1 min-h-[260px] flex items-center justify-center">
										<!-- CSS donut: zero JS chart dependency -->
										<div
											v-if="donutGradient"
											class="relative w-60 h-60 rounded-full"
											role="img"
											aria-label="Storage allocation by category"
											:style="{ background: donutGradient }"
										>
											<div class="absolute inset-8 rounded-full bg-neutral-50 dark:bg-neutral-900 flex flex-col items-center justify-center">
												<span class="text-lg font-bold text-neutral-900 dark:text-neutral-100">
													{{ formatBytes(treeSizeData.totalSize) }}
												</span>
												<span class="text-[10px] text-neutral-400 uppercase tracking-widest font-semibold">Total</span>
											</div>
										</div>
										<div v-else class="text-xs text-neutral-400">No categorizable files found</div>
									</div>
								</div>

								<!-- Category List & Progress Bars -->
								<div class="lg:col-span-7 p-5 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/30">
									<div class="flex items-center justify-between mb-4">
										<h3 class="text-xs font-bold uppercase tracking-wider text-neutral-500">File Categories Breakdown</h3>
										<span class="text-xs text-neutral-400">Sorted by size</span>
									</div>

									<div class="space-y-3">
										<div
											v-for="(cat, idx) in treeSizeData.byCategory"
											:key="cat.category"
											class="space-y-1.5"
										>
											<div class="flex items-center justify-between text-xs font-medium">
												<div class="flex items-center gap-2">
													<div
														class="w-3 h-3 rounded-full shrink-0"
														:style="{ backgroundColor: chartColors[idx % chartColors.length] }"
													/>
													<span class="text-neutral-800 dark:text-neutral-200">{{ cat.category }}</span>
													<span class="text-[11px] text-neutral-400 font-mono">({{ cat.count }} files)</span>
												</div>
												<div class="flex items-center gap-3 font-mono">
													<span class="font-bold">{{ formatBytes(cat.size) }}</span>
													<span class="text-neutral-400 text-[11px] w-12 text-right">{{ cat.percent }}%</span>
												</div>
											</div>

											<div class="h-2 w-full bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden">
												<div
													class="h-full rounded-full transition-all duration-500"
													:style="{
														width: `${Math.max(1, cat.percent)}%`,
														backgroundColor: chartColors[idx % chartColors.length]
													}"
												/>
											</div>
										</div>
									</div>
								</div>
							</div>

							<!-- Subfolders Tree Table -->
							<div class="rounded-2xl border border-neutral-200 dark:border-neutral-800 overflow-hidden">
								<div class="px-5 py-3 bg-neutral-100/70 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
									<div class="flex items-center gap-2">
										<UIcon name="lucide:folder-tree" class="w-4 h-4 text-primary-500" />
										<h3 class="text-xs font-bold uppercase tracking-wider text-neutral-700 dark:text-neutral-300">
											Subdirectories Ranked by Space Footprint
										</h3>
									</div>
									<span class="text-xs text-neutral-400 font-mono">{{ treeSizeData.subfolders.length }} folders</span>
								</div>

								<div class="overflow-x-auto max-h-72">
									<table class="w-full text-left text-xs">
										<thead class="bg-neutral-50 dark:bg-neutral-900/80 text-neutral-500 border-b border-neutral-200 dark:border-neutral-800 uppercase font-mono text-[10px]">
											<tr>
												<th class="py-2.5 px-4 font-semibold">Folder Name</th>
												<th class="py-2.5 px-4 font-semibold w-48">% Usage</th>
												<th class="py-2.5 px-4 font-semibold text-right">Size</th>
												<th class="py-2.5 px-4 font-semibold text-right">Files</th>
												<th class="py-2.5 px-4 font-semibold text-right">Subfolders</th>
												<th class="py-2.5 px-4 font-semibold text-center w-24">Action</th>
											</tr>
										</thead>
										<tbody class="divide-y divide-neutral-100 dark:divide-neutral-800/60 font-medium">
											<tr
												v-for="sub in treeSizeData.subfolders"
												:key="sub.path"
												class="hover:bg-neutral-50 dark:hover:bg-neutral-900/50 transition-colors"
											>
												<td
													class="py-2.5 px-4 flex items-center gap-2 cursor-pointer"
													title="Drill down"
													@click="openTreeSize(sub.path)"
												>
													<UIcon name="lucide:folder" class="w-4 h-4 text-amber-500 shrink-0" />
													<span class="font-semibold text-neutral-800 dark:text-neutral-200 truncate max-w-xs hover:underline">{{ sub.name }}</span>
												</td>
												<td class="py-2.5 px-4">
													<div class="flex items-center gap-2">
														<div class="flex-1 h-2 bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden">
															<div
																class="h-full bg-emerald-500 rounded-full"
																:style="{ width: `${Math.max(1, sub.percent)}%` }"
															/>
														</div>
														<span class="text-[11px] font-mono text-neutral-500 w-10 text-right">{{ sub.percent }}%</span>
													</div>
												</td>
												<td class="py-2.5 px-4 text-right font-mono font-bold">{{ formatBytes(sub.size) }}</td>
												<td class="py-2.5 px-4 text-right font-mono text-neutral-500">{{ sub.filesCount }}</td>
												<td class="py-2.5 px-4 text-right font-mono text-neutral-500">{{ sub.dirsCount }}</td>
												<td class="py-2.5 px-4 text-center">
													<UButton
														variant="subtle"
														color="neutral"
														size="xs"
														icon="lucide:folder-open"
														@click="openFolder(sub.path)"
													>
														Open
													</UButton>
												</td>
											</tr>
											<tr v-if="treeSizeData.subfolders.length === 0">
												<td colspan="6" class="py-8 text-center text-neutral-400">No subfolders detected</td>
											</tr>
										</tbody>
									</table>
								</div>
							</div>

							<!-- Top 30 Largest Files -->
							<div class="rounded-2xl border border-neutral-200 dark:border-neutral-800 overflow-hidden">
								<div class="px-5 py-3 bg-neutral-100/70 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between">
									<div class="flex items-center gap-2">
										<UIcon name="lucide:flame" class="w-4 h-4 text-rose-500" />
										<h3 class="text-xs font-bold uppercase tracking-wider text-neutral-700 dark:text-neutral-300">
											Top Largest Files (Disk Space Consumers)
										</h3>
									</div>
									<span class="text-xs text-neutral-400 font-mono">{{ treeSizeData.topLargestFiles.length }} largest files</span>
								</div>

								<div class="overflow-x-auto max-h-72">
									<table class="w-full text-left text-xs">
										<thead class="bg-neutral-50 dark:bg-neutral-900/80 text-neutral-500 border-b border-neutral-200 dark:border-neutral-800 uppercase font-mono text-[10px]">
											<tr>
												<th class="py-2.5 px-4 font-semibold">File Name</th>
												<th class="py-2.5 px-4 font-semibold">Location</th>
												<th class="py-2.5 px-4 font-semibold text-right">Size</th>
												<th class="py-2.5 px-4 font-semibold text-center w-28">Actions</th>
											</tr>
										</thead>
										<tbody class="divide-y divide-neutral-100 dark:divide-neutral-800/60 font-medium">
											<tr
												v-for="file in treeSizeData.topLargestFiles"
												:key="file.path"
												class="hover:bg-neutral-50 dark:hover:bg-neutral-900/50 transition-colors"
											>
												<td class="py-2 px-4 flex items-center gap-2">
													<UIcon name="lucide:file" class="w-4 h-4 text-blue-500 shrink-0" />
													<span class="font-medium text-neutral-800 dark:text-neutral-200 truncate max-w-xs">{{ file.name }}</span>
												</td>
												<td class="py-2 px-4 text-neutral-400 font-mono text-[11px] truncate max-w-sm">
													{{ file.path }}
												</td>
												<td class="py-2 px-4 text-right font-mono font-bold text-rose-600 dark:text-rose-400">
													{{ formatBytes(file.size) }}
												</td>
												<td class="py-2 px-4 text-center">
													<div class="flex items-center justify-center gap-1">
														<UButton
															variant="ghost"
															color="neutral"
															size="xs"
															icon="lucide:eye"
															title="Quick Peek"
															@click="quickPeekFile(file.path)"
														/>
														<UButton
															variant="ghost"
															color="neutral"
															size="xs"
															icon="lucide:external-link"
															title="Reveal in Explorer"
															@click="openFolder(getFolderOf(file.path))"
														/>
													</div>
												</td>
											</tr>
											<tr v-if="treeSizeData.topLargestFiles.length === 0">
												<td colspan="4" class="py-8 text-center text-neutral-400">No large files found</td>
											</tr>
										</tbody>
									</table>
								</div>
							</div>
						</div>
					</div>

					<!-- Footer -->
					<div class="px-6 py-3 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex items-center justify-between text-xs text-neutral-500">
						<div class="flex items-center gap-2">
							<UIcon name="lucide:info" class="w-4 h-4 text-neutral-400" />
							<span>Parallel Rust scan, cached: click a folder to drill down instantly. Rescan to refresh.</span>
						</div>
						<UButton
							variant="outline"
							color="neutral"
							size="sm"
							@click="isTreeSizeOpen = false"
						>
							Close
						</UButton>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script lang="ts" setup>
	const {
		isTreeSizeOpen,
		treeSizeData,
		treeSizeLoading,
		treeSizeProgress,
		currentPath,
		loadDirectory,
		openTreeSize,
		selectItem,
		togglePeek
	} = useExplorer();

	const chartColors = [
		'#10b981', // emerald
		'#3b82f6', // blue
		'#8b5cf6', // purple
		'#f59e0b', // amber
		'#ec4899', // pink
		'#06b6d4', // cyan
		'#64748b'  // slate
	];

	const chartData = computed(() => treeSizeData.value?.byCategory ?? []);

	// conic-gradient stops, same colors/order as the category list
	const donutGradient = computed(() => {
		const total = treeSizeData.value?.totalSize;
		if (!total || !chartData.value.length) return '';
		let at = 0;
		const stops = chartData.value.map((cat, i) => {
			const from = at;
			at += (cat.size / total) * 360;
			return `${chartColors[i % chartColors.length]} ${from}deg ${at}deg`;
		});
		return `conic-gradient(${stops.join(', ')})`;
	});

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
	}

	function getFolderOf(filePath: string): string {
		return filePath.replace(/[\\/][^\\/]+$/, '');
	}

	function openFolder(folderPath: string) {
		isTreeSizeOpen.value = false;
		loadDirectory(folderPath);
	}

	function quickPeekFile(filePath: string) {
		selectItem(filePath, false);
		togglePeek();
	}
</script>
