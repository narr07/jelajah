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
				v-if="isHashModalOpen && hashTarget"
				class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 md:p-8 select-none"
				tabindex="0"
				@keydown.esc="isHashModalOpen = false"
			>
				<!-- Independent Backdrop Layer (no backdrop-filter blur ghosting) -->
				<div class="absolute inset-0 bg-black/75" @click="isHashModalOpen = false" />

				<div
					class="relative z-10 w-full max-w-2xl flex flex-col rounded-2xl border border-neutral-200 dark:border-neutral-700 shadow-2xl overflow-hidden text-neutral-900 dark:text-neutral-100"
					style="background-color: #ffffff; isolation: isolate;"
				>
					<!-- Header Bar -->
					<div class="flex items-center justify-between px-6 py-4 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900">
						<div class="flex items-center gap-3 min-w-0">
							<div class="w-10 h-10 rounded-xl flex items-center justify-center bg-blue-500/10 text-blue-600 dark:text-blue-400">
								<UIcon name="lucide:fingerprint" class="w-6 h-6" />
							</div>
							<div class="min-w-0">
								<h2 class="text-base font-bold tracking-tight truncate">File Checksum & Integrity Hasher</h2>
								<div class="flex items-center gap-2 text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
									<span class="truncate max-w-md">{{ hashTarget.name }}</span>
									<span>•</span>
									<span class="font-mono">{{ formatBytes(hashTarget.size) }}</span>
								</div>
							</div>
						</div>

						<button
							class="p-2 rounded-lg text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 hover:bg-neutral-200/50 dark:hover:bg-neutral-800 transition-colors"
							title="Close (Esc)"
							@click="isHashModalOpen = false"
						>
							<UIcon name="lucide:x" class="w-5 h-5" />
						</button>
					</div>

					<!-- Body -->
					<div class="p-6 space-y-6 bg-white dark:bg-neutral-950 overflow-y-auto max-h-[75vh]">
						<!-- Loading Indicator -->
						<div v-if="hashLoading" class="py-12 flex flex-col items-center justify-center space-y-3">
							<UIcon name="lucide:loader-2" class="w-8 h-8 text-primary-500 animate-spin" />
							<div class="text-sm font-medium">Computing cryptographic checksums...</div>
							<div class="text-xs text-neutral-400 font-mono">MD5 • SHA-1 • SHA-256 • SHA-512</div>
						</div>

						<div v-else-if="hashResult" class="space-y-5">
							<!-- Verification / Compare Box -->
							<div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/80 dark:bg-neutral-900/40 space-y-2">
								<div class="flex items-center justify-between">
									<label class="text-xs font-bold uppercase tracking-wider text-neutral-600 dark:text-neutral-400 flex items-center gap-1.5">
										<UIcon name="lucide:shield-check" class="w-4 h-4 text-primary-500" />
										Verify against expected hash
									</label>
									<UButton
										variant="link"
										color="primary"
										size="xs"
										icon="lucide:clipboard"
										@click="pasteExpectedHash"
									>
										Paste from clipboard
									</UButton>
								</div>

								<div class="relative">
									<input
										v-model="expectedHash"
										type="text"
										placeholder="Paste hash here to verify integrity (e.g. from developer website)..."
										class="w-full px-3.5 py-2 text-xs font-mono rounded-lg border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 focus:outline-none focus:ring-2 focus:ring-primary-500"
									/>
									<button
										v-if="expectedHash"
										class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 text-xs"
										@click="expectedHash = ''"
									>
										<UIcon name="lucide:x" class="w-3.5 h-3.5" />
									</button>
								</div>

								<!-- Verification Result Badge -->
								<div v-if="matchStatus" class="pt-1">
									<div
										v-if="matchStatus.matched"
										class="flex items-center gap-2 p-2.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-300 dark:border-emerald-700 text-emerald-800 dark:text-emerald-300 text-xs font-semibold"
									>
										<UIcon name="lucide:check-circle-2" class="w-4 h-4 text-emerald-600 shrink-0" />
										<span>VERIFIED MATCH: Exact checksum matches {{ matchStatus.algo }}!</span>
									</div>
									<div
										v-else
										class="flex items-center gap-2 p-2.5 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-300 dark:border-rose-700 text-rose-800 dark:text-rose-300 text-xs font-semibold"
									>
										<UIcon name="lucide:alert-triangle" class="w-4 h-4 text-rose-600 shrink-0" />
										<span>CHECKSUM MISMATCH: Does not match any calculated hash.</span>
									</div>
								</div>
							</div>

							<!-- Hash algorithms list -->
							<div class="space-y-3">
								<!-- SHA-256 (Primary) -->
								<div class="p-3.5 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-900/30 space-y-1.5">
									<div class="flex items-center justify-between text-xs">
										<span class="font-bold text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5">
											<span class="w-2 h-2 rounded-full bg-blue-500" />
											SHA-256 (Industry Standard)
										</span>
										<UButton
											variant="ghost"
											color="neutral"
											size="xs"
											:icon="copiedAlgo === 'sha256' ? 'lucide:check' : 'lucide:copy'"
											@click="copyHash(hashResult.sha256, 'sha256')"
										>
											{{ copiedAlgo === 'sha256' ? 'Copied' : 'Copy' }}
										</UButton>
									</div>
									<div class="font-mono text-xs text-neutral-800 dark:text-neutral-200 bg-white dark:bg-neutral-950 p-2 rounded-md border border-neutral-200/80 dark:border-neutral-800 break-all select-all">
										{{ hashResult.sha256 }}
									</div>
								</div>

								<!-- MD5 -->
								<div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/30 dark:bg-neutral-900/20 space-y-1.5">
									<div class="flex items-center justify-between text-xs">
										<span class="font-bold text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5">
											<span class="w-2 h-2 rounded-full bg-amber-500" />
											MD5
										</span>
										<UButton
											variant="ghost"
											color="neutral"
											size="xs"
											:icon="copiedAlgo === 'md5' ? 'lucide:check' : 'lucide:copy'"
											@click="copyHash(hashResult.md5, 'md5')"
										>
											{{ copiedAlgo === 'md5' ? 'Copied' : 'Copy' }}
										</UButton>
									</div>
									<div class="font-mono text-xs text-neutral-800 dark:text-neutral-200 bg-white dark:bg-neutral-950 p-2 rounded-md border border-neutral-200/80 dark:border-neutral-800 break-all select-all">
										{{ hashResult.md5 }}
									</div>
								</div>

								<!-- SHA-1 -->
								<div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/30 dark:bg-neutral-900/20 space-y-1.5">
									<div class="flex items-center justify-between text-xs">
										<span class="font-bold text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5">
											<span class="w-2 h-2 rounded-full bg-purple-500" />
											SHA-1
										</span>
										<UButton
											variant="ghost"
											color="neutral"
											size="xs"
											:icon="copiedAlgo === 'sha1' ? 'lucide:check' : 'lucide:copy'"
											@click="copyHash(hashResult.sha1, 'sha1')"
										>
											{{ copiedAlgo === 'sha1' ? 'Copied' : 'Copy' }}
										</UButton>
									</div>
									<div class="font-mono text-xs text-neutral-800 dark:text-neutral-200 bg-white dark:bg-neutral-950 p-2 rounded-md border border-neutral-200/80 dark:border-neutral-800 break-all select-all">
										{{ hashResult.sha1 }}
									</div>
								</div>

								<!-- SHA-512 -->
								<div class="p-3 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/30 dark:bg-neutral-900/20 space-y-1.5">
									<div class="flex items-center justify-between text-xs">
										<span class="font-bold text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5">
											<span class="w-2 h-2 rounded-full bg-emerald-500" />
											SHA-512
										</span>
										<UButton
											variant="ghost"
											color="neutral"
											size="xs"
											:icon="copiedAlgo === 'sha512' ? 'lucide:check' : 'lucide:copy'"
											@click="copyHash(hashResult.sha512, 'sha512')"
										>
											{{ copiedAlgo === 'sha512' ? 'Copied' : 'Copy' }}
										</UButton>
									</div>
									<div class="font-mono text-[11px] text-neutral-800 dark:text-neutral-200 bg-white dark:bg-neutral-950 p-2 rounded-md border border-neutral-200/80 dark:border-neutral-800 break-all select-all">
										{{ hashResult.sha512 }}
									</div>
								</div>
							</div>
						</div>
					</div>

					<!-- Footer -->
					<div class="px-6 py-3 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-900 flex items-center justify-between text-xs">
						<span class="text-neutral-400">Hashes are calculated asynchronously using streaming crypto buffers.</span>
						<UButton
							variant="outline"
							color="neutral"
							size="sm"
							@click="isHashModalOpen = false"
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
		isHashModalOpen,
		hashTarget,
		hashResult,
		hashLoading
	} = useExplorer();

	const toast = useToast();
	const expectedHash = ref('');
	const copiedAlgo = ref<string | null>(null);

	const matchStatus = computed(() => {
		const clean = expectedHash.value.trim().toLowerCase();
		if (!clean || !hashResult.value) return null;

		if (clean === hashResult.value.sha256.toLowerCase()) return { matched: true, algo: 'SHA-256' };
		if (clean === hashResult.value.md5.toLowerCase()) return { matched: true, algo: 'MD5' };
		if (clean === hashResult.value.sha1.toLowerCase()) return { matched: true, algo: 'SHA-1' };
		if (clean === hashResult.value.sha512.toLowerCase()) return { matched: true, algo: 'SHA-512' };

		return { matched: false };
	});

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${Number.parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
	}

	async function copyHash(hash: string, algo: string) {
		try {
			await navigator.clipboard.writeText(hash);
			copiedAlgo.value = algo;
			toast.add({
				title: `${algo.toUpperCase()} Copied`,
				description: 'Checksum copied to clipboard',
				color: 'success'
			});
			setTimeout(() => {
				copiedAlgo.value = null;
			}, 2000);
		} catch {
			// Fallback
		}
	}

	async function pasteExpectedHash() {
		try {
			const text = await navigator.clipboard.readText();
			expectedHash.value = text.trim();
		} catch {
			toast.add({
				title: 'Clipboard Access',
				description: 'Please paste the hash into the input directly.',
				color: 'neutral'
			});
		}
	}
</script>
