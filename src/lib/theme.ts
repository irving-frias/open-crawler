export function resolveTheme(saved: string): 'dark' | 'light' {
  if (saved === 'dark') return 'dark';
  if (saved === 'light') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(saved: string) {
  document.documentElement.setAttribute('data-theme', resolveTheme(saved));
  localStorage.setItem('theme', saved);
}

export const UI_STYLES = ['classic', 'neumorph', 'clay', 'glass', 'brutal'] as const;
export type UiStyle = (typeof UI_STYLES)[number];

export function resolveUiStyle(saved: string): UiStyle {
  return (UI_STYLES as readonly string[]).includes(saved) ? (saved as UiStyle) : 'classic';
}

export function applyUiStyle(saved: string) {
  const style = resolveUiStyle(saved);
  document.documentElement.setAttribute('data-style', style);
  localStorage.setItem('ui-style', style);
}
