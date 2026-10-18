<template>
	<USeparator class="h-px">
		<img :src="useAsset('logo.svg')" alt="" class="size-6">
	</USeparator>

	<UFooter>
		<template #left>
			<p class="text-muted text-sm">
				Jelajah · MIT License · © {{ new Date().getFullYear() }} narr07
			</p>
		</template>

		<template #right>
			<p class="text-muted text-sm hidden sm:block">
				Site based on the <ULink to="https://github.com/nuxt-ui-templates/saas" target="_blank" class="underline">Nuxt UI SaaS template</ULink>
			</p>
			<UButton :to="REPO" target="_blank" icon="i-simple-icons-github" aria-label="Jelajah on GitHub" color="neutral" variant="ghost" />
		</template>
	</UFooter>
</template>
