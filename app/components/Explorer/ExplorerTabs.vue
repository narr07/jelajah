<template>
	<div class="flex items-center h-10 px-2 bg-neutral-100 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 select-none">
		<!-- Tabs scroll container -->
		<div class="flex items-center gap-1 overflow-x-auto no-scrollbar flex-1 h-full pt-1">
			<div
				v-for="tab in tabs"
				:key="tab.id"
				class="group relative flex items-center gap-2 px-3 h-8 rounded-t-lg text-xs font-medium cursor-pointer transition-colors max-w-[200px] min-w-[120px]"
				:class="[
					activeTabId === tab.id
						? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 shadow-sm border-t border-x border-neutral-200 dark:border-neutral-700'
						: 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/60 dark:hover:bg-neutral-800/60'
				]"
				@click="switchTab(tab.id)"
			>
				<UIcon name="lucide:folder" class="w-4 h-4 text-amber-500 shrink-0" />
				<span class="truncate flex-1">{{ tab.title }}</span>

				<!-- Close tab button -->
				<button
					v-if="tabs.length > 1"
					class="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-neutral-300 dark:hover:bg-neutral-700 transition-opacity"
					title="Close tab (Ctrl+W)"
					@click.stop="closeTab(tab.id)"
				>
					<UIcon name="lucide:x" class="w-3.5 h-3.5" />
				</button>
			</div>

			<!-- New tab button -->
			<button
				class="p-1.5 ml-1 rounded-md text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100 hover:bg-neutral-200 dark:hover:bg-neutral-800 transition-colors"
				title="New tab (Ctrl+T)"
				@click="addTab()"
			>
				<UIcon name="lucide:plus" class="w-4 h-4" />
			</button>
		</div>

		<!-- Window controls placeholder if needed / right controls -->
		<div class="flex items-center gap-1 pl-2">
			<SiteColorModeButton />
		</div>
	</div>
</template>

<script lang="ts" setup>
	const { tabs, activeTabId, switchTab, closeTab, addTab } = useExplorer();
</script>
