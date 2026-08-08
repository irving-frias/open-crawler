import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default tseslint.config(
  {
    ignores: [
      'build/**',
      'dist/**',
      'node_modules/**',
      '.svelte-kit/**',
      'src/lib/paraglide/**',
      'src/lib/components/ui/**',
      'vite.trace.config.ts',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      'src-tauri/data/**',
      'tools/**',
    ],
  },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs['flat/recommended'],
  prettier,
  {
    // Svelte files use `<script lang="ts">`; hand the script body to the
    // TypeScript parser (the svelte plugin's flat config already sets the
    // outer svelte-eslint-parser).
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: { parser: tseslint.parser },
    },
  },
  {
    // Svelte 5 runes files (.svelte.ts) are plain TypeScript.
    files: ['**/*.svelte.ts'],
    languageOptions: {
      parser: tseslint.parser,
    },
  },
  {
    files: ['**/*.{js,mjs,ts,svelte}'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      // The results table renders search-highlighted crawled titles through
      // `applyHighlight`, which HTML-escapes its input before inserting <mark>.
      '@typescript-eslint/no-explicit-any': 'off',
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // Desktop app: <a> links are external (crawled URLs) opened in a new tab;
      // there is no SvelteKit router navigation from them.
      'svelte/no-navigation-without-resolve': 'off',
      // Sets/Maps stored in $state are reassigned wholesale (not mutated), so
      // SvelteMap/SvelteSet are unnecessary.
      'svelte/prefer-svelte-reactivity': 'off',
    },
  }
);
