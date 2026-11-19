import { countOf } from '#lib/utils/plural.ts';

export type Stage = 'add' | 'remove';

export type PickCheck = boolean | 'mixed';

export function initialCheck(holding: number, total: number): PickCheck {
  if (holding <= 0) return false;
  return holding >= total ? true : 'mixed';
}

export function nextStage(holding: number, total: number, stage: Stage | undefined): Stage | null {
  const initial = initialCheck(holding, total);
  if (initial === true) return stage === 'remove' ? null : 'remove';
  if (initial === false) return stage === 'add' ? null : 'add';
  if (stage === undefined) return 'add';
  return stage === 'add' ? 'remove' : null;
}

export function shownCheck(holding: number, total: number, stage: Stage | undefined): PickCheck {
  if (stage === 'add') return true;
  if (stage === 'remove') return false;
  return initialCheck(holding, total);
}

export function stageNote(holding: number, total: number, stage: Stage | undefined): string | null {
  if (stage === 'add') return `Add ${countOf(total - holding, 'photo')}`;
  if (stage === 'remove') return `Remove from ${countOf(holding, 'photo')}`;
  if (holding <= 0) return null;
  return holding >= total ? `All ${countOf(total, 'photo')}` : `In ${holding} of ${total}`;
}

export function stageSummary(stages: ReadonlyMap<string, Stage>): {
  count: number;
  detail: string;
} {
  const added = [...stages.values()].filter((stage) => stage === 'add').length;
  const removed = stages.size - added;
  const parts = [
    added > 0 ? `add ${added}` : null,
    removed > 0 ? `remove ${removed}` : null
  ].filter((part) => part !== null);
  const detail = parts.join(' · ');
  return { count: stages.size, detail: detail.charAt(0).toUpperCase() + detail.slice(1) };
}
