export interface ChartTheme {
  text: string;
  textSecondary: string;
  textMuted: string;
  grid: string;
  accent: string;
  tooltipBg: string;
  tooltipBorder: string;
}

const FALLBACK: ChartTheme = {
  text: '#e1e3e6',
  textSecondary: '#a6a7ab',
  textMuted: '#5c5f66',
  grid: '#2e333c',
  accent: '#667eea',
  tooltipBg: '#1b1e24',
  tooltipBorder: '#2e333c',
};

function cssVar(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

export function getChartTheme(): ChartTheme {
  return {
    text: cssVar('--text', FALLBACK.text),
    textSecondary: cssVar('--text-secondary', FALLBACK.textSecondary),
    textMuted: cssVar('--text-muted', FALLBACK.textMuted),
    grid: cssVar('--border', FALLBACK.grid),
    accent: cssVar('--accent', FALLBACK.accent),
    tooltipBg: cssVar('--bg-card', FALLBACK.tooltipBg),
    tooltipBorder: cssVar('--border', FALLBACK.tooltipBorder),
  };
}

export function resolveColor(value: string | undefined): string {
  if (!value) return '';
  const match = /var\((--[\w-]+)(?:\s*,\s*([^)]+))?\)/.exec(value.trim());
  if (match) {
    const resolved = cssVar(match[1], match[2]?.trim() || '');
    if (resolved) return resolved;
  }
  return value;
}

export function fontFamily(): string {
  return getComputedStyle(document.body).fontFamily || '-apple-system, BlinkMacSystemFont, sans-serif';
}

export function watchTheme(callback: () => void): () => void {
  const observer = new MutationObserver(() => callback());
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['data-theme', 'class'],
  });
  return () => observer.disconnect();
}
