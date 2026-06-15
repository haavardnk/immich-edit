import { settings } from './settings.svelte';

class MetadataConsentStore {
  open = $state(false);
  private pending: ((value: boolean) => void) | null = null;

  gate = (): Promise<boolean> => {
    if (settings.metadataPushConsented) return Promise.resolve(true);
    if (this.pending) return Promise.resolve(false);
    this.open = true;
    return new Promise<boolean>((resolve) => {
      this.pending = resolve;
    });
  };

  confirm = (): void => {
    settings.setMetadataPushConsented(true);
    this.open = false;
    this.resolve(true);
  };

  cancel = (): void => {
    this.open = false;
    this.resolve(false);
  };

  reset = (): void => {
    settings.setMetadataPushConsented(false);
  };

  private resolve(value: boolean): void {
    const pending = this.pending;
    this.pending = null;
    pending?.(value);
  }
}

export const metadataConsent = new MetadataConsentStore();
