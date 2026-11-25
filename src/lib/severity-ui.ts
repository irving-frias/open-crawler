import type { BadgeVariant } from '#lib/components/ui/badge/badge.svelte';

/**
 * The app's single severity language (T16, UI_UX_PLAN §1.1): error (red),
 * warning (amber), info (blue), ok (green) — badge + icon + text, never
 * color alone, so a critical error is distinguishable from a notice
 * without reading the text.
 */

export type Severity = 'error' | 'warning' | 'info' | 'ok';

export interface SeverityMeta {
  /** Badge variant from the shared Badge component. */
  variant: BadgeVariant;
  /** Text glyph shown next to the label (icon, not emoji). */
  icon: string;
  /** Paraglide message key for the severity label. */
  labelKey: 'severity.error' | 'severity.warning' | 'severity.info' | 'severity.ok';
  /** CSS class for non-Badge uses (bars, dots, plain text). */
  className: 'severity-error' | 'severity-warning' | 'severity-info' | 'severity-ok';
}

const SEVERITY_META: Record<Severity, SeverityMeta> = {
  error: {
    variant: 'destructive',
    icon: '✕',
    labelKey: 'severity.error',
    className: 'severity-error',
  },
  warning: {
    variant: 'warning',
    icon: '!',
    labelKey: 'severity.warning',
    className: 'severity-warning',
  },
  info: {
    variant: 'info',
    icon: 'i',
    labelKey: 'severity.info',
    className: 'severity-info',
  },
  ok: {
    variant: 'success',
    icon: '✓',
    labelKey: 'severity.ok',
    className: 'severity-ok',
  },
};

/** Normalize any severity string to one of the four levels (unknown → info). */
export function severityMeta(severity: string): SeverityMeta {
  if (severity === 'error') return SEVERITY_META.error;
  if (severity === 'warning') return SEVERITY_META.warning;
  if (severity === 'ok') return SEVERITY_META.ok;
  return SEVERITY_META.info;
}

/** Badge variant for a severity, for call sites that render Badge directly. */
export function severityVariant(severity: string): BadgeVariant {
  return severityMeta(severity).variant;
}

/**
 * Map an HTTP status code to a severity level (T16): 2xx ok, 3xx info,
 * 4xx/5xx error. Used where status codes are shown as badges (Duplicates).
 */
export function statusSeverity(statusCode: number): Severity {
  if (statusCode >= 400) return 'error';
  if (statusCode >= 300) return 'info';
  return 'ok';
}
