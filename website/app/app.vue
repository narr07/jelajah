<script setup lang="ts">
const colorMode = useColorMode();

useHead({
	meta: [{ key: 'theme-color', name: 'theme-color', content: computed(() => colorMode.value === 'dark' ? '#06110f' : '#ffffff') }],
	htmlAttrs: { lang: 'en' }
});
</script>

<template>
	<UApp>
		<AppHeader />
		<UMain>
			<NuxtPage />
		</UMain>
		<AppFooter />
	</UApp>
</template>
