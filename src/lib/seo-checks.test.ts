import { afterEach, describe, expect, it } from 'vitest';
import { setLocale } from '#lib/paraglide/runtime.js';
import { localizeSeoCheck } from './seo-checks';

afterEach(async () => {
  await setLocale('en');
});

describe('localizeSeoCheck', () => {
  it('replaces the placeholders with the numbers of the English message', () => {
    const out = localizeSeoCheck(
      'img_dimensions',
      'Images with explicit dimensions: 3/10',
      'Add width and height to every image.',
      null
    );
    expect(out.message).toContain('3/10');
    expect(out.message).not.toContain('{0}');
  });

  it('prefers the numeric evidence over the numbers in the message', () => {
    const out = localizeSeoCheck(
      'img_dimensions',
      'Images with explicit dimensions: 3/10',
      'Add width and height to every image.',
      '4/20'
    );
    expect(out.message).toContain('4/20');
  });

  it('keeps the raw backend text for unknown check ids', () => {
    const out = localizeSeoCheck('a_check_from_the_future', 'Raw message', 'Raw guidance');
    expect(out).toEqual({ message: 'Raw message', guidance: 'Raw guidance' });
  });

  it('adds why/fix/expected for known checks', () => {
    const out = localizeSeoCheck('title_present', 'Page has no <title>', 'Add a title');
    expect(out.why).toBeTruthy();
    expect(out.fix).toBeTruthy();
    expect(out.expected).toBeTruthy();
  });

  it('falls back to the "no text" copy for readability when there is no prose', () => {
    const empty = localizeSeoCheck(
      'readability_score',
      'Not enough text to measure readability',
      'Simplify language'
    );
    expect(empty.message).toMatch(/text/i);

    const scored = localizeSeoCheck(
      'readability_score',
      'Flesch reading ease: 62',
      'Simplify language'
    );
    expect(scored.message).not.toBe(empty.message);
  });

  it('translates to Spanish when the locale changes', async () => {
    const en = localizeSeoCheck('title_present', 'Page has no <title>', 'Add a title');
    await setLocale('es');
    const es = localizeSeoCheck('title_present', 'Page has no <title>', 'Add a title');
    expect(es.message).not.toBe(en.message);
    expect(es.guidance).not.toBe(en.guidance);
  });
});
