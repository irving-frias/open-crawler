import { defineConfig } from 'vitest/config';

/**
 * Unit tests for the framework-agnostic frontend logic (no Svelte components).
 * The Svelte/Vite plugins are intentionally NOT loaded here: these tests run in
 * plain node and only import pure `.ts` modules, which keeps them fast and
 * avoids needing a DOM.
 */
export default defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      // Only modules covered by unit tests; UI components are exercised by
      // `svelte-check` + the manual QA pass, not here.
      include: ['src/lib/i18n-issues.ts', 'src/lib/seo-checks.ts', 'src/lib/seo-ui.ts'],
      thresholds: {
        lines: 70,
        functions: 70,
        statements: 70,
        branches: 70,
      },
    },
  },
});
