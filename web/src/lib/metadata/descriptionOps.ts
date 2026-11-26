import { browseTarget, setDescription } from '#lib/browse/assetMeta.ts';
import type { DescriptionChange } from '#lib/metadata/descriptions.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';
import { countFailures } from '#lib/utils/pool.ts';

const CONCURRENCY = 6;

async function write(
  changes: readonly DescriptionChange[],
  pick: (change: DescriptionChange) => string
): Promise<DescriptionChange[]> {
  const failed = new Set<string>();
  await countFailures(changes, CONCURRENCY, async (change) => {
    const outcome = await setDescription(
      browseTarget(change.id, () => {}),
      pick(change)
    );
    if (outcome !== 'saved') failed.add(change.id);
    return outcome === 'saved';
  });
  return changes.filter((change) => !failed.has(change.id));
}

async function undo(changes: readonly DescriptionChange[]): Promise<void> {
  const back = await write(changes, (change) => change.from);
  const failed = changes.length - back.length;
  if (failed > 0) {
    toasts.push('warn', `Could not undo the description of ${countOf(failed, 'photo')}`);
  }
}

export async function setDescriptions(changes: readonly DescriptionChange[]): Promise<boolean> {
  if (changes.length === 0) return true;
  if (!(await metadataConsent.gate())) return false;
  const written = await write(changes, (change) => change.to);
  const failed = changes.length - written.length;
  if (written.length === 0) {
    toasts.push('warn', `Could not change the description of ${countOf(failed, 'photo')}`);
    return false;
  }
  toasts.push(
    failed > 0 ? 'warn' : 'success',
    `Changed the description of ${countOf(written.length, 'photo')}`,
    {
      description: failed > 0 ? `${countOf(failed, 'photo')} could not be changed` : undefined,
      action: { label: 'Undo', run: () => void undo(written) }
    }
  );
  return true;
}
