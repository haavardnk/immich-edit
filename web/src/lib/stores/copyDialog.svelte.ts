import type { Edits } from '#lib/types/edits.ts';
import { clipboard, type ClipboardPayload } from '#lib/stores/clipboard.svelte.ts';
import {
  COPY_TOOLS,
  DEFAULT_COPY_SELECTION,
  selectedTools,
  selectionOf,
  withTools,
  type CopySelection,
  type CopyTool
} from '#lib/editor/copyTools.ts';

export type CopyDialogMode = 'copy' | 'paste';

class CopyDialogStore {
  open = $state(false);
  mode = $state<CopyDialogMode>('copy');
  selection = $state<CopySelection>({ ...DEFAULT_COPY_SELECTION });
  pasteSelection = $state<CopySelection>(selectionOf([]));
  available = $state.raw<readonly CopyTool[]>(COPY_TOOLS);
  source = $state.raw<Edits | null>(null);
  sourceName = $state<string | null>(null);
  touched = $state<CopyTool | null>(null);
  private onCopied: (() => void) | null = null;
  private onPaste: ((tools: CopySelection) => void) | null = null;

  get checked(): CopySelection {
    return this.mode === 'paste' ? this.pasteSelection : this.selection;
  }

  get count(): number {
    return selectedTools(this.checked).length;
  }

  show = (
    edits: Edits,
    onCopied?: () => void,
    sourceName: string | null = null,
    touched: CopyTool | null = null
  ): void => {
    this.mode = 'copy';
    this.available = COPY_TOOLS;
    this.source = structuredClone(edits) as Edits;
    this.sourceName = sourceName;
    this.touched = touched;
    this.onCopied = onCopied ?? null;
    this.onPaste = null;
    this.open = true;
  };

  showPaste = (
    payload: ClipboardPayload,
    onPaste: (tools: CopySelection) => void,
    sourceName: string | null = null
  ): void => {
    this.mode = 'paste';
    this.available = selectedTools(payload.tools);
    this.pasteSelection = { ...payload.tools };
    this.source = payload.edits;
    this.sourceName = sourceName;
    this.touched = payload.touched;
    this.onPaste = onPaste;
    this.onCopied = null;
    this.open = true;
  };

  setTools = (tools: readonly CopyTool[], value: boolean): void => {
    const shown = tools.filter((tool) => this.available.includes(tool));
    this.update(withTools(this.checked, shown, value));
  };

  setAll = (value: boolean): void => {
    this.setTools(this.available, value);
  };

  pickOnly = (tools: readonly CopyTool[]): void => {
    this.update(selectionOf(tools.filter((tool) => this.available.includes(tool))));
  };

  confirm = (): void => {
    if (!this.source || this.count === 0) return;
    if (this.mode === 'paste') {
      const onPaste = this.onPaste;
      const tools = { ...this.pasteSelection };
      this.close();
      onPaste?.(tools);
      return;
    }
    clipboard.copy(this.source, this.selection, this.sourceName, this.touched);
    const onCopied = this.onCopied;
    this.close();
    onCopied?.();
  };

  cancel = (): void => {
    this.close();
  };

  private update(next: CopySelection): void {
    if (this.mode === 'paste') this.pasteSelection = next;
    else this.selection = next;
  }

  private close(): void {
    this.open = false;
    this.source = null;
    this.onCopied = null;
    this.onPaste = null;
  }
}

export const copyDialog = new CopyDialogStore();
