import type { Edits } from '$lib/types/edits';
import { clipboard } from '$lib/stores/clipboard.svelte';
import { DEFAULT_COPY_SECTIONS, hasSelectedSections, type CopySections } from '$lib/copyPaste';

class CopyDialogStore {
  open = $state(false);
  sections = $state<CopySections>({ ...DEFAULT_COPY_SECTIONS });
  private source: Edits | null = null;
  private onCopied: (() => void) | null = null;

  show = (edits: Edits, onCopied?: () => void): void => {
    this.source = structuredClone(edits) as Edits;
    this.onCopied = onCopied ?? null;
    this.open = true;
  };

  confirm = (): void => {
    if (!this.source || !hasSelectedSections(this.sections)) return;
    clipboard.copy(this.source, this.sections);
    this.open = false;
    this.source = null;
    this.onCopied?.();
    this.onCopied = null;
  };

  cancel = (): void => {
    this.open = false;
    this.source = null;
    this.onCopied = null;
  };
}

export const copyDialog = new CopyDialogStore();
