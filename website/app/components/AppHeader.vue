<script setup lang="ts">
const items = [
	{ label: 'Features', to: '/#features' },
	{ label: 'PowerToys', to: '/#powertoys' },
	{ label: 'Docs', to: '/#docs' },
	{ label: 'Download', to: '/#download' }
];
</script>

<template>
	<UHeader>
		<template #left>
			<NuxtLink to="/" class="focus-visible:outline-3 outline-primary/25 rounded-md p-1 -ms-1" aria-label="Jelajah home">
				<AppLogo />
			</NuxtLink>
		</template>

		<UNavigationMenu :items="items" variant="link" />

		<template #right>
			<UColorModeButton />
			<UButton :to="REPO" target="_blank" icon="i-simple-icons-github" color="neutral" variant="ghost" aria-label="Jelajah on GitHub" />
			<UButton :to="DOWNLOAD" target="_blank" label="Download" icon="i-lucide-download" class="hidden lg:inline-flex" />
		</template>

		<template #body>
			<UNavigationMenu :items="items" orientation="vertical" class="-mx-2.5" />
			<USeparator class="my-6" />
			<UButton :to="DOWNLOAD" target="_blank" label="Download for Windows" icon="i-lucide-download" block />
		</template>
	</UHeader>
</template>
