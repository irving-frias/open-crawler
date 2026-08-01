import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
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
    tailwindcss(),
    sveltekit(),
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
