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
				v-if="isPowerRenameOpen"
				class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-10 select-none"
			>
				<!-- Independent Backdrop Layer -->
				<div class="absolute inset-0 bg-black/60 backdrop-blur-md" @click="isPowerRenameOpen = false" />

				<div class="relative z-10 w-full max-w-4xl max-h-[90vh] flex flex-col bg-white dark:bg-neutral-900 rounded-2xl border border-neutral-200 dark:border-neutral-800 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100">
					<!-- Header Bar -->
					<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
						<div class="flex items-center gap-3">
							<div class="w-8 h-8 rounded-lg flex items-center justify-center bg-blue-50 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400">
								<UIcon name="lucide:spell-check" class="w-5 h-5 shrink-0" />
							</div>
							<div>
								<h3 class="text-sm font-semibold leading-tight">PowerRename</h3>
								<p class="text-xs text-neutral-500 dark:text-neutral-400">
									Batch rename {{ targetItems.length }} items with search, regex & case transforms
								</p>
							</div>
						</div>
						<UButton
							variant="ghost"
							size="sm"
							icon="lucide:x"
							@click="isPowerRenameOpen = false"
						/>
					</div>

					<!-- Settings Controls Body -->
					<div class="p-6 border-b border-neutral-200 dark:border-neutral-800 space-y-4 bg-white dark:bg-neutral-900">
						<!-- Search & Replace Inputs Grid -->
						<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
							<div>
								<label class="block text-xs font-semibold mb-1 text-neutral-700 dark:text-neutral-300">
									Find Text
								</label>
								<UInput
									v-model="findText"
									placeholder="e.g. IMG_ or (\d{4})"
									icon="lucide:search"
									autofocus
								/>
							</div>
							<div>
								<label class="block text-xs font-semibold mb-1 text-neutral-700 dark:text-neutral-300">
									Replace With
								</label>
								<UInput
									v-model="replaceText"
									placeholder="e.g. Photo_ or $1"
									icon="lucide:replace"
								/>
							</div>
						</div>

						<!-- Options & Filters -->
						<div class="flex flex-wrap items-center gap-3 pt-1 text-xs">
							<!-- Regex -->
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-colors"
								:class="useRegex ? 'bg-primary-50 dark:bg-primary-950/50 border-primary-500 text-primary-600 dark:text-primary-400 font-semibold' : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-400'"
								@click="useRegex = !useRegex"
							>
								<UIcon name="lucide:regex" class="w-3.5 h-3.5" />
								<span>Use Regular Expression (RegEx)</span>
							</button>

							<!-- Match Case -->
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-colors"
								:class="matchCase ? 'bg-primary-50 dark:bg-primary-950/50 border-primary-500 text-primary-600 dark:text-primary-400 font-semibold' : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-400'"
								@click="matchCase = !matchCase"
							>
								<UIcon name="lucide:case-sensitive" class="w-3.5 h-3.5" />
								<span>Match Case</span>
							</button>

							<!-- Numbering Enum -->
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-colors"
								:class="enableNumbering ? 'bg-primary-50 dark:bg-primary-950/50 border-primary-500 text-primary-600 dark:text-primary-400 font-semibold' : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-400'"
								@click="enableNumbering = !enableNumbering"
							>
								<UIcon name="lucide:binary" class="w-3.5 h-3.5" />
								<span>Append Numbering Enum</span>
							</button>

							<!-- Target Scope Select -->
							<div class="flex items-center gap-1.5 ml-auto">
								<span class="text-neutral-500">Apply to:</span>
								<select
									v-model="targetScope"
									class="px-2.5 py-1 text-xs rounded-lg border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800 focus:outline-none"
								>
									<option value="nameOnly">Filename only</option>
									<option value="all">Filename + Extension</option>
									<option value="extOnly">Extension only</option>
								</select>
							</div>

							<!-- Case Transform -->
							<div class="flex items-center gap-1.5">
								<span class="text-neutral-500">Case:</span>
								<select
									v-model="caseTransform"
									class="px-2.5 py-1 text-xs rounded-lg border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800 focus:outline-none"
								>
									<option value="none">No change</option>
									<option value="lower">lowercase</option>
									<option value="upper">UPPERCASE</option>
									<option value="title">Title Case</option>
									<option value="camel">camelCase</option>
									<option value="kebab">kebab-case</option>
									<option value="snake">snake_case</option>
								</select>
							</div>
						</div>
					</div>

					<!-- Real-Time Diff Preview Table -->
					<div class="flex-1 overflow-auto max-h-[46vh] divide-y divide-neutral-100 dark:divide-neutral-800/60 bg-neutral-50 dark:bg-neutral-950">
						<table class="w-full text-left border-collapse text-xs">
							<thead class="sticky top-0 bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 font-semibold select-none shadow-xs z-10">
								<tr>
									<th class="py-2.5 px-4 w-1/2">Original Name</th>
									<th class="py-2.5 px-4 w-1/2">Renamed Result</th>
									<th class="py-2.5 px-4 w-28 text-center">Status</th>
								</tr>
							</thead>
							<tbody>
								<tr
									v-for="(item, idx) in previewList"
									:key="idx"
									class="hover:bg-neutral-100/60 dark:hover:bg-neutral-800/40 transition-colors"
								>
									<td class="py-2 px-4 font-mono truncate max-w-xs text-neutral-600 dark:text-neutral-300">
										{{ item.originalName }}
									</td>
									<td class="py-2 px-4 font-mono font-medium truncate max-w-xs" :class="item.changed ? 'text-primary-600 dark:text-primary-400' : 'text-neutral-400 dark:text-neutral-600'">
										{{ item.newName }}
									</td>
									<td class="py-2 px-4 text-center">
										<span
											v-if="item.hasConflict"
											class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-red-100 dark:bg-red-950 text-red-600 dark:text-red-400"
										>
											Duplicate
										</span>
										<span
											v-else-if="item.changed"
											class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-100 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-400"
										>
											Renamed
										</span>
										<span
											v-else
											class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-neutral-100 dark:bg-neutral-800 text-neutral-400"
										>
											Unchanged
										</span>
									</td>
								</tr>
							</tbody>
						</table>
					</div>

					<!-- Footer Actions Bar -->
					<div class="px-6 py-4 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex items-center justify-between text-xs">
						<div class="text-neutral-500 dark:text-neutral-400">
							<span class="font-semibold text-neutral-900 dark:text-neutral-100">{{ changedCount }}</span> of {{ targetItems.length }} files will be modified
						</div>

						<div class="flex items-center gap-3">
							<UButton
								variant="ghost"
								color="neutral"
								@click="isPowerRenameOpen = false"
							>
								Cancel
							</UButton>
							<UButton
								color="primary"
								icon="lucide:check"
								:disabled="changedCount === 0 || hasAnyConflict"
								@click="applyRename"
							>
								Apply Rename ({{ changedCount }})
							</UButton>
						</div>
					</div>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script lang="ts" setup>
	import type { FileItem } from '~/composables/useExplorerApi';

	const {
		isPowerRenameOpen,
		items,
		selectedPaths,
		batchRename
	} = useExplorer();

	const findText = ref('');
	const replaceText = ref('');
	const useRegex = ref(false);
	const matchCase = ref(false);
	const enableNumbering = ref(false);
	const targetScope = ref<'nameOnly' | 'all' | 'extOnly'>('nameOnly');
	const caseTransform = ref<'none' | 'lower' | 'upper' | 'title' | 'camel' | 'kebab' | 'snake'>('none');

	// Selected items if any, otherwise all items
	const targetItems = computed<FileItem[]>(() => {
		if (selectedPaths.value.size > 0) {
			return items.value.filter(i => selectedPaths.value.has(i.path));
		}
		return items.value;
	});

	// Transform case helper
	const applyCaseTransform = (str: string, type: string) => {
		switch (type) {
			case 'lower':
				return str.toLowerCase();
			case 'upper':
				return str.toUpperCase();
			case 'title':
				return str.replace(/\b\w/g, c => c.toUpperCase());
			case 'camel':
				return str.replace(/[-_](\w)/g, (_, c) => c.toUpperCase());
			case 'kebab':
				return str.replace(/\s+/g, '-').replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase();
			case 'snake':
				return str.replace(/\s+/g, '_').replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase();
			default:
				return str;
		}
	};

	// Real-time preview calculation
	const previewList = computed(() => {
		const list: { originalName: string; newName: string; changed: boolean; hasConflict: boolean; oldPath: string; newPath: string }[] = [];
		const newNamesSet = new Set<string>();

		targetItems.value.forEach((item, index) => {
			const originalName = item.name;
			const lastDot = originalName.lastIndexOf('.');
			let baseName = lastDot > 0 ? originalName.slice(0, lastDot) : originalName;
			let ext = lastDot > 0 ? originalName.slice(lastDot) : '';

			let targetPart = '';
			if (targetScope.value === 'all') {
				targetPart = originalName;
			} else if (targetScope.value === 'extOnly') {
				targetPart = ext.startsWith('.') ? ext.slice(1) : ext;
			} else {
				targetPart = baseName;
			}

			// Apply Search & Replace
			if (findText.value) {
				try {
					if (useRegex.value) {
						const flags = matchCase.value ? 'g' : 'gi';
						const regex = new RegExp(findText.value, flags);
						targetPart = targetPart.replace(regex, replaceText.value);
					} else {
						if (matchCase.value) {
							targetPart = targetPart.replaceAll(findText.value, replaceText.value);
						} else {
							const regex = new RegExp(findText.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
							targetPart = targetPart.replace(regex, replaceText.value);
						}
					}
				} catch {
					// Invalid regex syntax - keep original
				}
			}

			// Apply Case Transform
			if (caseTransform.value !== 'none') {
				targetPart = applyCaseTransform(targetPart, caseTransform.value);
			}

			// Append numbering if enabled
			if (enableNumbering.value) {
				const num = String(index + 1).padStart(2, '0');
				targetPart = `${targetPart}_${num}`;
			}

			// Reconstruct final filename
			let finalName = '';
			if (targetScope.value === 'all') {
				finalName = targetPart;
			} else if (targetScope.value === 'extOnly') {
				finalName = targetPart ? `${baseName}.${targetPart}` : baseName;
			} else {
				finalName = `${targetPart}${ext}`;
			}

			const isChanged = finalName !== originalName;
			const hasConflict = newNamesSet.has(finalName.toLowerCase());
			newNamesSet.add(finalName.toLowerCase());

			// Compute new full path
			const dir = item.path.substring(0, Math.max(item.path.lastIndexOf('/'), item.path.lastIndexOf('\\')));
			const separator = item.path.includes('\\') ? '\\' : '/';
			const newFullPath = `${dir}${separator}${finalName}`;

			list.push({
				originalName,
				newName: finalName,
				changed: isChanged,
				hasConflict,
				oldPath: item.path,
				newPath: newFullPath
			});
		});

		return list;
	});

	const changedCount = computed(() => {
		return previewList.value.filter(i => i.changed).length;
	});

	const hasAnyConflict = computed(() => {
		return previewList.value.some(i => i.hasConflict);
	});

	const applyRename = async () => {
		const toRename = previewList.value
			.filter(i => i.changed && !i.hasConflict)
			.map(i => ({ oldPath: i.oldPath, newPath: i.newPath }));

		if (toRename.length > 0) {
			await batchRename(toRename);
		}
	};
</script>
