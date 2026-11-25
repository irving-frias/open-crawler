import { afterEach, describe, expect, it } from 'vitest';
import { setLocale } from '#lib/paraglide/runtime.js';
import {
  parseIssueParams,
  translateIssueExpected,
  translateIssueFix,
  translateIssueMessage,
  translateIssueName,
  translateIssueWhy,
  translateSeverity,
} from './i18n-issues';

afterEach(async () => {
  await setLocale('en');
});

describe('parseIssueParams', () => {
  it('extracts the <h1> count from multiple_h1', () => {
    expect(parseIssueParams('Found 3 <h1> tags', 'multiple_h1')).toEqual({ count: '3' });
  });

  it('extracts the <main> count from multiple_main', () => {
    expect(
      parseIssueParams('Page has 2 <main> elements; only one is recommended', 'multiple_main')
    ).toEqual({ count: '2' });
  });

  it('extracts the duplicated id from duplicate_id', () => {
    expect(parseIssueParams('Duplicate id attribute "hero"', 'duplicate_id')).toEqual({
      id: 'hero',
    });
  });

  it('extracts both levels from heading_skip', () => {
    expect(parseIssueParams('heading jump h2 to h4', 'heading_skip')).toEqual({
      prev: '2',
      level: '4',
    });
  });

  it('extracts child and parent from nesting issues', () => {
    // The parser formats these as "<child> inside <parent>".
    expect(parseIssueParams('<div> inside <span>', 'invalid_nesting')).toEqual({
      child: 'div',
      parent: 'span',
    });
    expect(parseIssueParams('<p> inside <a>', 'context_nesting')).toEqual({
      child: 'p',
      parent: 'a',
    });
  });

  it('extracts the tag from empty-element issues', () => {
    expect(parseIssueParams('Empty <li> element', 'empty_heading')).toEqual({ tag: 'li' });
    expect(parseIssueParams('Empty <ul> element', 'empty_content_tag')).toEqual({ tag: 'ul' });
  });

  it('returns no params when the message does not match', () => {
    expect(parseIssueParams('something else', 'multiple_h1')).toEqual({});
    expect(parseIssueParams('', 'heading_skip')).toEqual({});
    expect(parseIssueParams('whatever', 'missing_title')).toEqual({});
  });
});

describe('translateIssue*', () => {
  it('localizes known issue types', () => {
    expect(translateIssueName('missing_title')).toMatch(/title/i);
    expect(translateIssueMessage('missing_title')).not.toBe('');
    expect(translateIssueWhy('missing_title')).not.toBe('');
    expect(translateIssueFix('missing_title')).not.toBe('');
    expect(translateIssueExpected('missing_title')).not.toBe('');
  });

  it('interpolates params in messages that need them', () => {
    expect(translateIssueMessage('multiple_h1', { count: 3 })).toContain('3');
    expect(translateIssueMessage('multiple_main', { count: 2 })).toContain('2');
    expect(translateIssueMessage('duplicate_id', { id: 'hero' })).toContain('hero');
    expect(translateIssueMessage('heading_skip', { prev: 2, level: 4 })).toContain('4');
    expect(translateIssueMessage('invalid_nesting', { child: 'div', parent: 'span' })).toContain(
      'div'
    );
  });

  it('falls back to a humanized id for unknown names and empty strings elsewhere', () => {
    expect(translateIssueName('some_new_issue')).toBe('some new issue');
    expect(translateIssueMessage('some_new_issue')).toBe('');
    expect(translateIssueWhy('some_new_issue')).toBe('');
    expect(translateIssueFix('some_new_issue')).toBe('');
    expect(translateIssueExpected('some_new_issue')).toBe('');
  });

  it('switches language with the paraglide locale', async () => {
    const en = translateIssueName('missing_title');
    await setLocale('es');
    const es = translateIssueName('missing_title');
    expect(es).not.toBe(en);
  });
});

describe('translateSeverity', () => {
  it('localizes known severities and passes unknown ones through', () => {
    expect(translateSeverity('error')).not.toBe('error');
    expect(translateSeverity('warning')).not.toBe('warning');
    expect(translateSeverity('info')).not.toBe('info');
    expect(translateSeverity('critical')).toBe('critical');
  });
});

describe('i18n coverage of parser issue types', () => {
  // Regression guard for the 7 types that used to be emitted by the parser
  // without any translation (they rendered as "some new issue" + empty text).
  const previouslyMissing = [
    'aria_current_nav',
    'button_no_name',
    'duplicate_id',
    'form_no_accessible_name',
    'multiple_main',
    'section_no_accessible_name',
    'skip_link_missing',
  ];

  it.each(previouslyMissing)('%s has name, message, why, fix and expected', (id) => {
    expect(translateIssueName(id)).not.toBe(id.replace(/_/g, ' '));
    expect(translateIssueMessage(id)).not.toBe('');
    expect(translateIssueWhy(id)).not.toBe('');
    expect(translateIssueFix(id)).not.toBe('');
    expect(translateIssueExpected(id)).not.toBe('');
  });
});
