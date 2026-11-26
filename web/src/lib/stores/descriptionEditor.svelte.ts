import { sourceId } from '#lib/browse/assetKey.ts';
import { setDescriptions } from '#lib/metadata/descriptionOps.ts';
import { resolveAssets } from '#lib/browse/assetLookup.ts';
import { ownsAsset } from '#lib/browse/ownership.ts';
import {
  descriptionChanges,
  descriptionValues,
  type DescriptionValue
} from '#lib/metadata/descriptions.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

class DescriptionEditor {
  assets = $state.raw<AssetSummary[] | null>(null);
  text = $state('');
  saving = $state(false);

  readonly values: DescriptionValue[] = $derived(descriptionValues(this.assets ?? []));
  readonly mixed = $derived(this.values.length > 1);

  open = (assets: readonly AssetSummary[]): void => {
    const owned = assets.filter((asset) => ownsAsset(asset));
    if (owned.length === 0) {
      toasts.push('warn', 'Another Immich user owns these photos');
      return;
    }
    if (owned.length < assets.length) {
      toasts.push(
        'info',
        `${countOf(assets.length - owned.length, 'photo')} owned by someone else left out`
      );
    }
    const sources = new Set<string>();
    const unique = owned.filter((asset) => {
      const source = sourceId(asset.id);
      if (sources.has(source)) return false;
      sources.add(source);
      return true;
    });
    this.assets = unique;
    const values = descriptionValues(unique);
    this.text = values.length === 1 ? (values[0]?.text ?? '') : '';
  };

  openIds = async (ids: readonly string[]): Promise<void> => {
    try {
      this.open(await resolveAssets(ids));
    } catch (error) {
      toasts.fail('Description', error);
    }
  };

  close = (): void => {
    if (!this.saving) this.assets = null;
  };

  save = async (clear = false): Promise<void> => {
    const assets = this.assets;
    if (!assets || this.saving) return;
    const text = clear ? '' : this.text.trim();
    if (!clear && text === '' && this.mixed) {
      this.assets = null;
      return;
    }
    this.saving = true;
    try {
      if (await setDescriptions(descriptionChanges(assets, text))) this.assets = null;
    } finally {
      this.saving = false;
    }
  };
}

export const descriptionEditor = new DescriptionEditor();
