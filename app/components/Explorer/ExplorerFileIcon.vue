<template>
	<span ref="el" class="inline-flex items-center justify-center shrink-0">
		<img v-if="src" :src="src" alt="" draggable="false" class="w-full h-full object-contain">
		<UIcon v-else :name="fallback" class="w-full h-full" :class="isDir ? 'text-amber-500' : 'text-neutral-500 dark:text-neutral-400'" />
	</span>
</template>

<script lang="ts" setup>
	import { useElementVisibility } from '@vueuse/core';

	// Windows' own icon for the item (same as explorer.exe), loaded only once it scrolls into view.
	const props = defineProps<{ path: string; isDir: boolean; fallback: string }>();

	const el = ref<HTMLElement | null>(null);
	const visible = useElementVisibility(el);
	const src = ref<string | null>(null);

	// One request per icon type for the whole session (exe/lnk/ico/url have their own icon).
	const OWN_ICON = /\.(exe|ico|lnk|url|cur|ani|msc|appref-ms)$/i;
	const cache = (globalThis as any).__nxIconCache ??= new Map<string, Promise<string | null>>();
	const api = useExplorerApi();

	watch([visible, () => props.path], async ([isVisible]) => {
		if (!isVisible) return;
		const key = props.isDir ? '<dir>' : OWN_ICON.test(props.path) ? props.path : (props.path.match(/\.[^.\\/]+$/)?.[0] ?? '').toLowerCase();
		if (!cache.has(key)) cache.set(key, api.fileIcon(props.path, props.isDir).catch(() => null));
		src.value = await cache.get(key);
	}, { immediate: true });
</script>
