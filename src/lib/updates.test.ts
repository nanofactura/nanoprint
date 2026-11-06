import { describe, expect, it } from 'vitest';
import type { UpdateStatus } from '$lib/agent';
import { waitingVersion } from './updates.svelte';

const view = (automatic: boolean, status: UpdateStatus) => ({ automatic, status });

describe('waitingVersion', () => {
  it('shows a version found while automatic updates are off', () => {
    for (const state of ['available', 'downloading', 'ready'] as const) {
      expect(waitingVersion(view(false, { state, version: '0.4.0' }))).toBe('0.4.0');
    }
  });

  it('shows nothing while automatic updates install it by themselves', () => {
    for (const state of ['available', 'downloading', 'ready'] as const) {
      expect(waitingVersion(view(true, { state, version: '0.4.0' }))).toBeUndefined();
    }
  });

  it('shows nothing without a newer version', () => {
    expect(waitingVersion(view(false, { state: 'upToDate', checkedAt: 1 }))).toBeUndefined();
    expect(waitingVersion(view(false, { state: 'checking' }))).toBeUndefined();
    expect(waitingVersion(undefined)).toBeUndefined();
  });
});
