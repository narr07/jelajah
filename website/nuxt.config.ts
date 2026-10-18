// Landing page for Jelajah, based on the Nuxt UI SaaS template (MIT, nuxt-ui-templates/saas).
// Fully static: `bun run generate` -> .output/public. For GitHub Pages under /jelajah/,
// build with NUXT_APP_BASE_URL=/jelajah/.
export default defineNuxtConfig({
	modules: ['@nuxt/ui'],

	css: ['~/assets/css/main.css'],

	app: {
		head: {
			link: [
				{ rel: 'icon', type: 'image/svg+xml', href: 'logo.svg' },
				{ rel: 'alternate icon', href: 'favicon.ico' }
			]
		}
	},

	devtools: { enabled: false },

	compatibilityDate: '2026-06-30'
});
