import { m } from '$lib/paraglide/messages.js';

type IssueParams = Record<string, string | number>;

const issueNames: Record<string, () => string> = {
  missing_html_lang: () => m["issue.missing_html_lang.name"](),
  missing_title: () => m["issue.missing_title.name"](),
  missing_meta_description: () => m["issue.missing_meta_description.name"](),
  missing_canonical: () => m["issue.missing_canonical.name"](),
  missing_main: () => m["issue.missing_main.name"](),
  missing_header: () => m["issue.missing_header.name"](),
  missing_footer: () => m["issue.missing_footer.name"](),
  missing_nav: () => m["issue.missing_nav.name"](),
  missing_h1: () => m["issue.missing_h1.name"](),
  multiple_h1: () => m["issue.multiple_h1.name"](),
  heading_skip: () => m["issue.heading_skip.name"](),
  img_no_alt: () => m["issue.img_no_alt.name"](),
  input_no_id: () => m["issue.input_no_id.name"](),
  input_no_label: () => m["issue.input_no_label.name"](),
  empty_link_text: () => m["issue.empty_link_text.name"](),
  missing_aria: () => m["issue.missing_aria.name"](),
  invalid_nesting: () => m["issue.invalid_nesting.name"](),
  context_nesting: () => m["issue.context_nesting.name"](),
};

const issueMessages: Record<string, (params?: IssueParams) => string> = {
  missing_html_lang: () => m["issue.missing_html_lang.message"](),
  missing_title: () => m["issue.missing_title.message"](),
  missing_meta_description: () => m["issue.missing_meta_description.message"](),
  missing_canonical: () => m["issue.missing_canonical.message"](),
  missing_main: () => m["issue.missing_main.message"](),
  missing_header: () => m["issue.missing_header.message"](),
  missing_footer: () => m["issue.missing_footer.message"](),
  missing_nav: () => m["issue.missing_nav.message"](),
  missing_h1: () => m["issue.missing_h1.message"](),
  multiple_h1: (p) => m["issue.multiple_h1.message"]({ count: String(p?.count ?? 0) }),
  heading_skip: (p) => m["issue.heading_skip.message"]({ prev: String(p?.prev ?? 0), level: String(p?.level ?? 0) }),
  img_no_alt: (p) => m["issue.img_no_alt.message"]({ count: String(p?.count ?? 0) }),
  input_no_id: (p) => m["issue.input_no_id.message"]({ count: String(p?.count ?? 0) }),
  input_no_label: (p) => m["issue.input_no_label.message"]({ count: String(p?.count ?? 0) }),
  empty_link_text: (p) => m["issue.empty_link_text.message"]({ count: String(p?.count ?? 0) }),
  missing_aria: (p) => m["issue.missing_aria.message"]({ count: String(p?.count ?? 0) }),
  invalid_nesting: (p) => m["issue.invalid_nesting.message"]({ child: String(p?.child ?? ''), parent: String(p?.parent ?? '') }),
  context_nesting: (p) => m["issue.context_nesting.message"]({ child: String(p?.child ?? ''), parent: String(p?.parent ?? '') }),
};

export function translateIssueName(issueType: string): string {
  return issueNames[issueType]?.() ?? issueType.replace(/_/g, ' ');
}

export function translateIssueMessage(issueType: string, params?: IssueParams): string {
  return issueMessages[issueType]?.(params) ?? '';
}

export function parseIssueParams(message: string, issueType: string): IssueParams {
  const params: IssueParams = {};
  if (issueType === 'multiple_h1') {
    const m = message.match(/\((\d+)\)/);
    if (m) params.count = m[1];
  } else if (issueType === 'heading_skip') {
    const m = message.match(/h(\d+)\s+to\s+h(\d+)/);
    if (m) { params.prev = m[1]; params.level = m[2]; }
  } else if (['img_no_alt', 'input_no_id', 'input_no_label', 'empty_link_text', 'missing_aria'].includes(issueType)) {
    const m = message.match(/^(\d+)/);
    if (m) params.count = m[1];
  } else if (issueType === 'invalid_nesting' || issueType === 'context_nesting') {
    const childM = message.match(/<(\w+)>/);
    const parentM = message.match(/<(\w+)>.*?<(\w+)>/);
    if (childM) params.child = childM[1];
    if (parentM) params.parent = parentM[2];
  }
  return params;
}

const severityLabels: Record<string, () => string> = {
  error: () => m["severity.error"](),
  warning: () => m["severity.warning"](),
  info: () => m["severity.info"](),
};

export function translateSeverity(severity: string): string {
  return severityLabels[severity]?.() ?? severity;
}
