import type { ParamRange } from '#lib/editor/params/types.ts';

export type ValueEntryTarget = {
  label: string;
  range: ParamRange;
  start: number;
  resetValue: number;
  owner: HTMLElement;
  live: (value: number) => void;
  finish: (commit: boolean) => void;
};

export class ValueEntryStore {
  target = $state.raw<ValueEntryTarget | null>(null);

  open = (target: ValueEntryTarget): void => {
    this.close(true);
    this.target = target;
  };

  close = (commit: boolean): void => {
    const target = this.target;
    if (!target) return;
    this.target = null;
    target.finish(commit);
  };

  release = (target: ValueEntryTarget): void => {
    if (this.target === target) this.close(true);
  };
}

export const valueEntry = new ValueEntryStore();
