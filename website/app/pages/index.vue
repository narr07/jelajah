<script setup lang="ts">
const title = 'Jelajah: a fast, light file explorer for Windows';
const description = 'Explorer you already know, with Everything-style instant search, a built-in TreeSize and the PowerToys file tools. Free and open source.';
useSeoMeta({ title, ogTitle: title, description, ogDescription: description });

const stats = [
	{ value: '< 10 ms', label: 'filename search across 2.7M files' },
	{ value: '23 s', label: 'to index every drive (NTFS MFT)' },
	{ value: '9.5 s', label: 'TreeSize scan of 700k files' },
	{ value: '0.1 s', label: 'from launch to search-ready' }
];

const searchHits = [
	{ icon: 'i-lucide-file-spreadsheet', color: 'text-emerald-600', name: 'Laporan Keuangan 2026.xlsx', path: 'D:\\Document\\Keuangan' },
	{ icon: 'i-lucide-file-text', color: 'text-sky-600', name: 'laporan-tahunan-2026.docx', path: 'C:\\Users\\you\\Documents' },
	{ icon: 'i-lucide-file', color: 'text-rose-500', name: 'Laporan Q1 2026.pdf', path: 'E:\\Arsip\\2026' }
];

const menu = [
	{ icon: 'i-lucide-external-link', label: 'Open', kbd: 'Enter' },
	{ icon: 'i-lucide-app-window', label: 'Open with', kbd: '›' },
	{ icon: 'i-lucide-scan-text', label: 'Copy Text from Image', kbd: '' },
	{ icon: 'i-lucide-info', label: 'Properties', kbd: 'Alt+Enter' },
	{ icon: 'i-lucide-square-arrow-out-up-right', label: 'Show more options', kbd: 'Shift+F10' }
];

const folders = [
	{ name: 'Videos', size: '48.2 GB', pct: 100 },
	{ name: 'Projects', size: '21.7 GB', pct: 45 },
	{ name: 'Downloads', size: '12.9 GB', pct: 27 },
	{ name: 'Pictures', size: '6.4 GB', pct: 13 }
];

const tools = [
	{ title: 'Peek', icon: 'i-lucide-eye', description: 'Press Space to look inside any file, Quick Look style. Office, PDF, code, images, video. Space again to close.' },
	{ title: 'PowerRename', icon: 'i-lucide-text-cursor-input', description: 'Batch rename with regex, casing and numbering, with a live before/after preview.' },
	{ title: 'Image Resizer', icon: 'i-lucide-image-down', description: 'Resize and convert to WebP, JPEG or PNG with presets or custom sizes.' },
	{ title: 'File Locksmith', icon: 'i-lucide-lock-keyhole', description: 'See which process is holding a file open, straight from Windows Restart Manager.' },
	{ title: 'New+', icon: 'i-lucide-file-plus-2', description: 'Your own file and folder templates, right in the New menu.' },
	{ title: 'Text Extractor', icon: 'i-lucide-scan-text', description: 'Copy the text out of any image with the OCR engine built into Windows.' },
	{ title: 'Command Palette', icon: 'i-lucide-command', description: 'Ctrl+P to jump to drives, folders and every action without touching the mouse.' },
	{ title: 'Advanced Paste', icon: 'i-lucide-clipboard-paste', description: 'Ctrl+Shift+V turns clipboard text or a screenshot into a file in the current folder.' },
	{ title: 'File Hasher', icon: 'i-lucide-fingerprint', description: 'MD5, SHA-1, SHA-256 and SHA-512 at once, checked against your clipboard.' }
];

const steps = [
	{ title: '1. Install', icon: 'i-lucide-download', description: 'Run the installer from the latest release. Jelajah installs for all users, like other system tools.' },
	{ title: '2. Search right away', icon: 'i-lucide-search', description: 'The filename index builds in the background on first launch and is ready instantly every launch after.' },
	{ title: '3. Optional: Fast Index', icon: 'i-lucide-rocket', description: 'Ctrl+P → "Toggle Fast Index". One admin prompt, then every drive is re-indexed from the NTFS MFT in seconds.' }
];

const shortcuts = [
	{ keys: ['Space'], label: 'Peek (again to close)' },
	{ keys: ['Ctrl', 'P'], label: 'Command Palette' },
	{ keys: ['Ctrl', 'T'], label: 'New tab' },
	{ keys: ['Ctrl', 'W'], label: 'Close tab' },
	{ keys: ['F2'], label: 'Rename' },
	{ keys: ['Ctrl', 'M'], label: 'PowerRename' },
	{ keys: ['Del'], label: 'Move to Recycle Bin' },
	{ keys: ['Shift', 'Del'], label: 'Delete permanently' },
	{ keys: ['Alt', 'Enter'], label: 'Properties' },
	{ keys: ['Shift', 'F10'], label: 'Windows context menu' },
	{ keys: ['Alt', 'T'], label: 'TreeSize' },
	{ keys: ['F6'], label: 'Dual pane' },
	{ keys: ['Alt', '←'], label: 'Back (or mouse back button)' },
	{ keys: ['Ctrl', 'Shift', 'V'], label: 'Paste as file' }
];
</script>

<template>
	<div>
		<UPageHero
			headline="Free & open source · Windows 10 / 11"
			description="The Explorer you already know, with Everything-style instant search, a built-in TreeSize and the PowerToys file tools. Rust under the hood, so it stays fast and light."
			:links="[
				{ label: 'Download for Windows', icon: 'i-lucide-download', to: DOWNLOAD, target: '_blank', size: 'xl' },
				{ label: 'View on GitHub', icon: 'i-simple-icons-github', to: REPO, target: '_blank', size: 'xl', color: 'neutral', variant: 'subtle' }
			]"
		>
			<template #top>
				<HeroBackground />
			</template>
			<template #title>
				Explore your files at <span class="text-primary">light speed</span>
			</template>

			<AppWindowMock />
		</UPageHero>

		<UContainer class="pb-8 sm:pb-16">
			<div class="grid grid-cols-2 lg:grid-cols-4 gap-px rounded-xl overflow-hidden border border-default bg-(--ui-border)">
				<div v-for="s in stats" :key="s.label" class="bg-default p-5 sm:p-6">
					<div class="text-2xl sm:text-3xl font-bold text-primary">{{ s.value }}</div>
					<div class="mt-1 text-sm text-muted">{{ s.label }}</div>
				</div>
			</div>
		</UContainer>

		<UPageSection
			id="features"
			headline="Search"
			title="Find any file before you finish typing"
			description="Every name on every drive lives in a compact in-memory index, so results land in milliseconds and stay current as files change."
			orientation="horizontal"
			:features="[
				{ title: 'Every drive, always live', description: 'Indexed once, then followed in real time through the same change notifications Explorer uses.', icon: 'i-lucide-hard-drive' },
				{ title: 'Fast Index from the MFT', description: 'Opt in once and Jelajah reads the NTFS master file table like Everything does. The app itself never runs as admin.', icon: 'i-lucide-rocket' },
				{ title: 'Search inside files too', description: 'A trigram index (tgrep) answers text searches across your documents and code.', icon: 'i-lucide-text-search' }
			]"
		>
			<UPageCard variant="subtle" class="shadow-xl shadow-primary/5">
				<div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-default border border-default">
					<UIcon name="i-lucide-search" class="size-4 text-muted" />
					<span class="text-sm">laporan 2026</span>
					<UBadge variant="subtle" class="ms-auto">13 results · 4 ms</UBadge>
				</div>
				<ul class="mt-3 space-y-1">
					<li v-for="h in searchHits" :key="h.name" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-elevated">
						<UIcon :name="h.icon" class="size-5 shrink-0" :class="h.color" />
						<div class="min-w-0">
							<div class="text-sm font-medium truncate">{{ h.name }}</div>
							<div class="text-xs text-muted truncate">{{ h.path }}</div>
						</div>
					</li>
				</ul>
			</UPageCard>
		</UPageSection>

		<UPageSection
			headline="Explorer parity"
			title="Feels like Explorer, because it uses Windows itself"
			description="Copy, move and delete go through the Windows shell. Icons, the context menu, Open with and Properties are the real ones."
			orientation="horizontal"
			reverse
			:features="[
				{ title: 'Recycle Bin and native dialogs', description: 'Delete goes to the Recycle Bin. Copies show the Windows progress and conflict dialogs, across drives too.', icon: 'i-lucide-trash-2' },
				{ title: 'Your shell extensions keep working', description: 'Show more options opens the full Windows context menu: 7-Zip, VS Code and the rest.', icon: 'i-lucide-menu' },
				{ title: 'Shortcuts you already know', description: 'F2, Del, Alt+Enter, Ctrl+T, mouse back and forward buttons, and Open in Terminal.', icon: 'i-lucide-keyboard' }
			]"
		>
			<UPageCard variant="subtle" class="shadow-xl shadow-primary/5 max-w-sm mx-auto w-full">
				<ul class="space-y-0.5 text-sm">
					<li v-for="m in menu" :key="m.label" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-elevated">
						<UIcon :name="m.icon" class="size-4 text-muted" />
						<span class="flex-1">{{ m.label }}</span>
						<span class="text-xs text-dimmed">{{ m.kbd }}</span>
					</li>
				</ul>
			</UPageCard>
		</UPageSection>

		<UPageSection
			headline="TreeSize"
			title="See where your disk space went"
			description="A full parallel scan of every folder, cached so drilling into a subfolder is instant."
			orientation="horizontal"
			:features="[
				{ title: 'Full depth, every core', description: 'No depth limit. Work is split at every folder level, so one giant folder never stalls the scan.', icon: 'i-lucide-cpu' },
				{ title: 'Largest files and file types', description: 'The biggest files and a breakdown by type, for any folder you click.', icon: 'i-lucide-chart-pie' }
			]"
		>
			<UPageCard variant="subtle" class="shadow-xl shadow-primary/5">
				<div class="text-sm font-medium mb-3">C:\Users\you · 89.2 GB</div>
				<div class="space-y-3">
					<div v-for="f in folders" :key="f.name">
						<div class="flex justify-between text-xs mb-1">
							<span class="flex items-center gap-1.5"><UIcon name="i-lucide-folder" class="size-3.5 text-amber-500" />{{ f.name }}</span>
							<span class="text-muted">{{ f.size }}</span>
						</div>
						<div class="h-2 rounded-full bg-accented overflow-hidden">
							<div class="h-full rounded-full bg-linear-to-r from-emerald-400 to-teal-600" :style="{ width: `${f.pct}%` }" />
						</div>
					</div>
				</div>
			</UPageCard>
		</UPageSection>

		<UPageSection
			id="powertoys"
			headline="PowerToys, built in"
			title="The file tools from Microsoft PowerToys"
			description="Every PowerToys module that works on files, without installing anything else."
		>
			<UPageGrid>
				<UPageCard v-for="t in tools" :key="t.title" v-bind="t" spotlight spotlight-color="primary" />
			</UPageGrid>
		</UPageSection>

		<UPageSection
			id="docs"
			headline="Docs"
			title="Get started in a minute"
			description="Nothing to configure. These are the few things worth knowing."
		>
			<UPageGrid class="lg:grid-cols-3">
				<UPageCard v-for="s in steps" :key="s.title" v-bind="s" variant="subtle" />
			</UPageGrid>

			<div class="mt-10">
				<h3 class="text-lg font-semibold text-highlighted mb-4">Keyboard shortcuts</h3>
				<div class="grid sm:grid-cols-2 gap-x-8">
					<div v-for="k in shortcuts" :key="k.label" class="flex items-center justify-between gap-4 py-2.5 border-b border-default">
						<span class="text-sm text-muted">{{ k.label }}</span>
						<span class="flex items-center gap-1 shrink-0">
							<UKbd v-for="key in k.keys" :key="key" :value="key" />
						</span>
					</div>
				</div>
				<p class="mt-6 text-sm text-muted">
					Full feature list and roadmap: <ULink :to="`${REPO}/blob/main/PRD.md`" target="_blank" class="text-primary underline">PRD.md</ULink>
					· Source and build steps: <ULink :to="REPO" target="_blank" class="text-primary underline">README</ULink>
				</p>
			</div>
		</UPageSection>

		<USeparator />

		<UPageCTA
			id="download"
			title="Download Jelajah"
			description="Windows 10 and 11, 64-bit. Installer (.exe) or MSI from the latest GitHub release. Free, MIT licensed."
			:links="[
				{ label: 'Download latest release', icon: 'i-lucide-download', to: DOWNLOAD, target: '_blank', size: 'xl' },
				{ label: 'Source on GitHub', icon: 'i-simple-icons-github', to: REPO, target: '_blank', size: 'xl', color: 'neutral', variant: 'subtle' }
			]"
			variant="naked"
			class="overflow-hidden"
		>
			<LazyStarsBg />
		</UPageCTA>
	</div>
</template>
