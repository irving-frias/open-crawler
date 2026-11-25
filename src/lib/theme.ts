export function resolveTheme(saved: string): 'dark' | 'light' {
  if (saved === 'dark') return 'dark';
  if (saved === 'light') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(saved: string) {
  document.documentElement.setAttribute('data-theme', resolveTheme(saved));
  localStorage.setItem('theme', saved);
}

export function resolveDensity(saved: string): 'comfortable' | 'compact' {
  return saved === 'compact' ? 'compact' : 'comfortable';
}

/** Density mode (T19, 1.9): compact shrinks row paddings app-wide. */
export function applyDensity(saved: string) {
  document.documentElement.setAttribute('data-density', resolveDensity(saved));
  localStorage.setItem('density', saved);
}
