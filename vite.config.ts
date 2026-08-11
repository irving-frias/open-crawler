import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig, type DevEnvironment } from 'vite';

export default defineConfig({
  plugins: [
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/lib/paraglide',
      emitTsDeclarations: true,
      strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
    }),
    tailwindcss(),
    sveltekit(),
    {
      // vite-plugin-svelte exposes a component's compiled <style> as a virtual
      // module (`File.svelte?svelte&type=style&lang.css`) served from an
      // in-memory compile cache. The cache is emptied on every dev-server
      // restart (config change, dependency re-optimization, ...). After such a
      // restart the browser may re-request a component's style module before
      // that component's JS transform has repopulated the cache (e.g. lazily
      // imported tab components). The svelte plugin's load()
      // then returns undefined, Vite falls back to serving the RAW .svelte
      // source as CSS, and @tailwindcss/vite crashes with "Invalid declaration:
      // ..." on the <script> block.
      // Intercept those cache misses: serve empty CSS instead of the raw file,
      // and warm the cache by transforming the component's JS module so the
      // next request gets the real compiled CSS.
      name: 'svelte-virtual-css-fallback',
      enforce: 'post',
      apply: 'serve',
      load(id) {
        if (/\.svelte\?[^#]*&type=style&lang\.css/.test(id)) {
          const env = this.environment as DevEnvironment;
          const jsUrl = id.split('?')[0];
          env.transformRequest(jsUrl)
            .then(() => {
              const mod = env.moduleGraph.getModuleById(id);
              if (mod) env.moduleGraph.invalidateModule(mod);
            })
            .catch(() => {});
          return { code: '' };
        }
      },
    },
  ],
  resolve: {
    conditions: ['module', 'browser', 'development|production', 'svelte'],
  },
  server: {
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  clearScreen: false,
  build: {
    cssCodeSplit: true,
  },
});
