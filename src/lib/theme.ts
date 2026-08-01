export function resolveTheme(saved: string): 'dark' | 'light' {
  if (saved === 'dark') return 'dark';
  if (saved === 'light') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(saved: string) {
  document.documentElement.setAttribute('data-theme', resolveTheme(saved));
  localStorage.setItem('theme', saved);
}
