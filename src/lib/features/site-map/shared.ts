import { m } from '$lib/paraglide/messages.js';

export type StatusClass = '2xx' | '3xx' | '4xx' | '5xx' | 'blocked' | 'unknown';

export function statusClass(code: number | null, blocked: boolean | null): StatusClass {
  if (blocked) return 'blocked';
  if (code == null) return 'unknown';
  if (code < 300) return '2xx';
  if (code < 400) return '3xx';
  if (code < 500) return '4xx';
  return '5xx';
}

export function statusLabel(sc: StatusClass): string {
  switch (sc) {
    case '2xx':
      return m['graph.filter_2xx']();
    case '3xx':
      return m['graph.filter_3xx']();
    case '4xx':
      return m['graph.filter_4xx']();
    case '5xx':
      return m['graph.filter_5xx']();
    case 'blocked':
      return m['graph.filter_blocked']();
    case 'unknown':
      return m['graph.filter_unknown']();
  }
}

export type StatusVariant = 'default' | 'warning' | 'destructive';

export function statusVariant(code: number | null): StatusVariant {
  if (code == null) return 'default';
  if (code >= 400) return 'destructive';
  if (code >= 300) return 'warning';
  return 'default';
}

export function cssVar(name: string, fallback: string): string {
  try {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
  } catch {
    return fallback;
  }
}

export function hexToRgba(hex: string): [number, number, number, number] {
  let h = hex.replace('#', '');
  if (h.length === 3)
    h = h
      .split('')
      .map((c) => c + c)
      .join('');
  if (h.length < 6) return [255, 255, 255, 255];
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
    h.length >= 8 ? parseInt(h.slice(6, 8), 16) : 255,
  ];
}

// The sphere shader shades with ambient+diffuse light (~65% of the base color),
// so the raw palette is brightened to render vivid on screen.
export function brightenRgb(r: number, g: number, b: number, f: number): [number, number, number] {
  return [
    Math.min(255, Math.round(r * f)),
    Math.min(255, Math.round(g * f)),
    Math.min(255, Math.round(b * f)),
  ];
}

export function shortLabel(text: string, max = 24): string {
  const trimmed = text.trim();
  if (trimmed.length <= max) return trimmed;
  return trimmed.slice(0, max - 1) + '…';
}

export const nodeFill: Record<StatusClass, string> = {
  '2xx': cssVar('--success', '#51cf66'),
  '3xx': cssVar('--warning', '#ffd43b'),
  '4xx': cssVar('--danger', '#ff6b6b'),
  '5xx': cssVar('--danger', '#ff6b6b'),
  blocked: cssVar('--info', '#74c0fc'),
  unknown: cssVar('--text-muted', '#6b7079'),
};

export const followEdgeRgba = hexToRgba(cssVar('--border-muted', '#3d4450'));
export const nofollowEdgeRgba = hexToRgba(cssVar('--text-muted', '#6b7079'));

export function legendColor(sc: StatusClass): string {
  switch (sc) {
    case '2xx':
      return cssVar('--success', '#51cf66');
    case '3xx':
      return cssVar('--warning', '#ffd43b');
    case '4xx':
    case '5xx':
      return cssVar('--danger', '#ff6b6b');
    case 'blocked':
      return cssVar('--info', '#74c0fc');
    case 'unknown':
      return cssVar('--text-muted', '#6b7079');
  }
}

export function formatBytes(bytes: number | null): string {
  if (bytes == null) return '—';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function formatMs(ms: number | null): string {
  if (ms == null) return '—';
  return `${ms} ms`;
}

// Common ISO 639-1 codes used as URL path prefixes for translated pages.
export const LANGUAGE_CODES = new Set([
  'en',
  'es',
  'fr',
  'de',
  'it',
  'pt',
  'nl',
  'pl',
  'ru',
  'uk',
  'tr',
  'sv',
  'no',
  'da',
  'fi',
  'cs',
  'sk',
  'hu',
  'ro',
  'bg',
  'el',
  'he',
  'ar',
  'hi',
  'ur',
  'fa',
  'th',
  'vi',
  'id',
  'ms',
  'ja',
  'zh',
  'ko',
  'ca',
  'eu',
  'gl',
  'hr',
  'sr',
  'sl',
  'lt',
  'lv',
  'et',
  'is',
  'mt',
  'ga',
  'cy',
  'sq',
  'mk',
  'bs',
  'az',
  'hy',
  'ka',
  'kk',
  'uz',
  'mn',
  'ne',
  'si',
  'ta',
  'te',
  'kn',
  'ml',
  'bn',
  'pa',
  'gu',
  'mr',
  'sw',
  'af',
  'am',
  'yo',
  'ig',
  'ha',
  'zu',
  'xh',
  'my',
  'km',
  'lo',
  'be',
  'lb',
  'br',
  'fy',
  'sc',
  'oc',
]);

export function segmentsOf(url: string): string[] {
  let path = '/';
  try {
    path = new URL(url).pathname;
  } catch {
    // keep fallback below
  }
  if (path.length > 1 && path.endsWith('/')) path = path.slice(0, -1);
  const segs: string[] = [];
  for (const s of path.split('/')) {
    if (!s) continue;
    try {
      segs.push(decodeURIComponent(s));
    } catch {
      segs.push(s);
    }
  }
  return segs;
}

export function isLangSegment(seg: string): boolean {
  return LANGUAGE_CODES.has(seg);
}

export function languageOf(url: string): string | null {
  const segs = segmentsOf(url);
  return segs.length > 0 && LANGUAGE_CODES.has(segs[0]) ? segs[0] : null;
}
