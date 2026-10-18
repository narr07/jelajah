<template>
	<aside class="w-56 shrink-0 h-full overflow-y-auto bg-neutral-50/50 dark:bg-neutral-900/50 border-r border-neutral-200 dark:border-neutral-800 p-2 text-xs select-none">
		<!-- Quick Access Section -->
		<div class="mb-4">
			<div class="px-2 py-1 text-[11px] font-semibold tracking-wider text-neutral-400 dark:text-neutral-500 uppercase">
				Quick access
			</div>
			<div class="space-y-0.5 mt-1">
				<button
					v-for="qa in quickAccess"
					:key="qa.id"
					type="button"
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left transition-colors font-medium truncate"
					:class="[
						currentPath.toLowerCase() === qa.path.toLowerCase()
							? 'bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-semibold shadow-xs'
							: 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60'
					]"
					@click="loadDirectory(qa.path)"
				>
					<UIcon :name="qa.icon" class="w-4 h-4 text-blue-500 shrink-0" />
					<span class="truncate">{{ qa.name }}</span>
				</button>
			</div>
		</div>

		<!-- This PC / Drives Section -->
		<div>
			<div class="px-2 py-1 text-[11px] font-semibold tracking-wider text-neutral-400 dark:text-neutral-500 uppercase flex items-center justify-between">
				<span>This PC</span>
				<UIcon name="lucide:monitor" class="w-3.5 h-3.5 text-neutral-400" />
			</div>
			<div class="space-y-0.5 mt-1">
				<button
					v-for="drive in drives"
					:key="drive.letter"
					type="button"
					class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-left transition-colors font-medium truncate"
					:class="[
						currentPath.toLowerCase().startsWith(drive.path.toLowerCase())
							? 'bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 font-semibold shadow-xs'
							: 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60'
					]"
					@click="loadDirectory(drive.path)"
				>
					<UIcon name="lucide:hard-drive" class="w-4 h-4 text-neutral-500 dark:text-neutral-400 shrink-0" />
					<div class="flex-1 min-w-0">
						<div class="truncate">{{ drive.label }}</div>
						<!-- Storage bar indicator if info available -->
						<div v-if="drive.total_bytes && drive.free_bytes" class="w-full h-1 bg-neutral-200 dark:bg-neutral-700 rounded-full mt-1 overflow-hidden">
							<div
								class="h-full bg-blue-500 rounded-full"
								:style="{ width: `${Math.round(((drive.total_bytes - drive.free_bytes) / drive.total_bytes) * 100)}%` }"
							/>
						</div>
					</div>
				</button>
			</div>
		</div>
	</aside>
</template>

<script lang="ts" setup>
	const { currentPath, quickAccess, drives, loadDirectory } = useExplorer();
</script>
