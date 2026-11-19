import type { Edits } from '#lib/types/edits.ts';
import { clipboard } from '#lib/stores/clipboard.svelte.ts';
import {
  DEFAULT_COPY_SECTIONS,
  DEVELOP_KEYS,
  allSections,
  hasSelectedSections,
  type CopySections,
  type SectionKey
} from '#lib/editor/copyPaste.ts';

class CopyDialogStore {
  open = $state(false);
  sections = $state<CopySections>({ ...DEFAULT_COPY_SECTIONS });
  private source: Edits | null = null;
  private sourceName: string | null = null;
  private onCopied: (() => void) | null = null;

  show = (edits: Edits, onCopied?: () => void, sourceName: string | null = null): void => {
    this.source = structuredClone(edits) as Edits;
    this.sourceName = sourceName;
    this.onCopied = onCopied ?? null;
    this.open = true;
  };

  setSection = (key: SectionKey, value: boolean): void => {
    this.sections = { ...this.sections, [key]: value };
  };

  setDevelop = (value: boolean): void => {
    const next = { ...this.sections };
    for (const k of DEVELOP_KEYS) next[k] = value;
    this.sections = next;
  };

  setAll = (value: boolean): void => {
    this.sections = allSections(value);
  };

  confirm = (): void => {
    if (!this.source || !hasSelectedSections(this.sections)) return;
    clipboard.copy(this.source, this.sections, this.sourceName);
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
