/** Public file URL that respects app.baseURL (e.g. /jelajah on GitHub Pages). */
export const useAsset = (file: string) => `${useRuntimeConfig().app.baseURL.replace(/\/?$/, '/')}${file}`;
