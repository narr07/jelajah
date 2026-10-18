<script setup lang="ts">
// A light HTML sketch of the Jelajah window for the hero (no screenshot to keep in sync).
const files = [
	{ icon: 'i-lucide-folder', color: 'text-amber-500', name: 'Projects', meta: 'File folder', size: '' },
	{ icon: 'i-lucide-file-spreadsheet', color: 'text-emerald-600', name: 'Laporan Keuangan 2026.xlsx', meta: 'Excel', size: '156 KB' },
	{ icon: 'i-lucide-file-text', color: 'text-sky-600', name: 'Proposal Jelajah.docx', meta: 'Word', size: '84 KB' },
	{ icon: 'i-lucide-image', color: 'text-violet-500', name: 'cover-photo.png', meta: 'PNG image', size: '2.4 MB' },
	{ icon: 'i-lucide-file-video', color: 'text-rose-500', name: 'demo.mp4', meta: 'MP4 video', size: '48 MB' },
	{ icon: 'i-lucide-file-archive', color: 'text-neutral-500', name: 'backup-2026.zip', meta: 'ZIP archive', size: '1.2 GB' }
];
const places = [
	{ icon: 'i-lucide-house', label: 'Home' },
	{ icon: 'i-lucide-monitor', label: 'Desktop' },
	{ icon: 'i-lucide-download', label: 'Downloads' },
	{ icon: 'i-lucide-file-text', label: 'Documents', active: true },
	{ icon: 'i-lucide-hard-drive', label: 'Local Disk (C:)' }
];
</script>

<template>
	<div class="rounded-xl border border-default bg-default shadow-2xl shadow-primary/10 overflow-hidden text-left text-xs select-none" aria-hidden="true">
		<!-- tabs -->
		<div class="flex items-center gap-1 px-3 pt-2 bg-muted border-b border-default">
			<div class="flex items-center gap-2 px-3 py-1.5 rounded-t-lg bg-default border-x border-t border-default">
				<UIcon name="i-lucide-folder" class="size-3.5 text-amber-500" />
				<span class="font-medium">Documents</span>
			</div>
			<div class="flex items-center gap-2 px-3 py-1.5 text-muted">
				<UIcon name="i-lucide-folder" class="size-3.5 text-amber-500" />
				<span>Downloads</span>
			</div>
			<UIcon name="i-lucide-plus" class="size-3.5 text-muted ms-1" />
		</div>
		<!-- address + search -->
		<div class="flex items-center gap-2 px-3 py-2 border-b border-default">
			<UIcon name="i-lucide-arrow-left" class="size-4 text-muted" />
			<UIcon name="i-lucide-arrow-right" class="size-4 text-dimmed" />
			<UIcon name="i-lucide-arrow-up" class="size-4 text-muted" />
			<div class="flex-1 px-2.5 py-1 rounded-md bg-muted text-muted truncate">This PC › Documents</div>
			<div class="hidden sm:flex items-center gap-1.5 w-44 px-2.5 py-1 rounded-md bg-muted text-muted">
				<UIcon name="i-lucide-search" class="size-3.5" />
				<span>laporan 2026</span>
				<UBadge size="sm" variant="subtle" class="ms-auto">4 ms</UBadge>
			</div>
		</div>
		<div class="flex">
			<!-- navigation pane -->
			<div class="hidden md:block w-44 p-2 border-e border-default space-y-0.5">
				<div
					v-for="p in places"
					:key="p.label"
					class="flex items-center gap-2 px-2 py-1.5 rounded-md"
					:class="p.active ? 'bg-primary/10 text-primary font-medium' : 'text-muted'"
				>
					<UIcon :name="p.icon" class="size-3.5" />
					{{ p.label }}
				</div>
			</div>
			<!-- details view -->
			<div class="flex-1 min-w-0">
				<div class="grid grid-cols-[1fr_auto] sm:grid-cols-[1fr_7rem_5rem] gap-2 px-3 py-1.5 text-dimmed border-b border-default">
					<span>Name</span><span class="hidden sm:block">Type</span><span class="text-right">Size</span>
				</div>
				<div
					v-for="(f, i) in files"
					:key="f.name"
					class="grid grid-cols-[1fr_auto] sm:grid-cols-[1fr_7rem_5rem] gap-2 px-3 py-1.5 items-center"
					:class="i === 1 ? 'bg-primary/10' : ''"
				>
					<span class="flex items-center gap-2 min-w-0">
						<UIcon :name="f.icon" class="size-4 shrink-0" :class="f.color" />
						<span class="truncate text-default">{{ f.name }}</span>
					</span>
					<span class="hidden sm:block text-muted truncate">{{ f.meta }}</span>
					<span class="text-right text-muted">{{ f.size }}</span>
				</div>
			</div>
		</div>
		<!-- status bar -->
		<div class="flex items-center justify-between px-3 py-1.5 border-t border-default bg-muted text-muted">
			<span>6 items · 1 selected</span>
			<span class="flex items-center gap-1.5"><UKbd size="sm">Space</UKbd> Peek</span>
		</div>
	</div>
</template>
