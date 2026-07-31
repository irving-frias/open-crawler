import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/lib/paraglide',
      emitTsDeclarations: true,
      strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
    }),
    sveltekit(),
  ],
  server: {
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  clearScreen: false,
});
