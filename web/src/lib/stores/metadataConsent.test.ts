import { describe, it, expect, beforeEach } from 'vitest';
import { metadataConsent } from './metadataConsent.svelte';
import { settings } from './settings.svelte';

describe('metadataConsent', () => {
  beforeEach(() => metadataConsent.reset());

  it('resolves immediately when already consented', async () => {
    settings.setMetadataPushConsented(true);
    await expect(metadataConsent.gate()).resolves.toBe(true);
    expect(metadataConsent.open).toBe(false);
  });

  it('opens and resolves true on confirm', async () => {
    const p = metadataConsent.gate();
    expect(metadataConsent.open).toBe(true);
    metadataConsent.confirm();
    await expect(p).resolves.toBe(true);
    expect(settings.metadataPushConsented).toBe(true);
    expect(metadataConsent.open).toBe(false);
  });

  it('opens and resolves false on cancel', async () => {
    metadataConsent.reset();
    const p = metadataConsent.gate();
    expect(metadataConsent.open).toBe(true);
    metadataConsent.cancel();
    await expect(p).resolves.toBe(false);
    expect(settings.metadataPushConsented).toBe(false);
  });
});
