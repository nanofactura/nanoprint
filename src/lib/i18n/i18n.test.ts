import { describe, expect, it } from 'vitest';
import en from '$locales/en.json';
import es from '$locales/es.json';
import { format, type MessageKey } from './index.svelte';

const placeholders = (template: string) =>
  [...template.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe('locales', () => {
  it('define the same keys', () => {
    expect(Object.keys(es).sort()).toEqual(Object.keys(en).sort());
  });

  it('use the same placeholders in every translation', () => {
    for (const key of Object.keys(en) as MessageKey[]) {
      expect(placeholders(es[key]), key).toEqual(placeholders(en[key]));
    }
  });
});

describe('format', () => {
  it('replaces known placeholders', () => {
    expect(format('Version {version}', { version: '1.2.3' })).toBe('Version 1.2.3');
  });

  it('leaves unknown placeholders untouched', () => {
    expect(format('Hello {name}', {})).toBe('Hello {name}');
  });
});
