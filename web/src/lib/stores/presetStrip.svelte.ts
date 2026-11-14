import type { ApplyPresetOptions } from '#lib/api/jobs.ts';
import type { Preset } from '#lib/api/presets.ts';
import type { PillText } from '#lib/editor/pill.ts';
import { defaultApplyOptions } from '#lib/edits/applyPreset.ts';
import { LOOK_AMOUNT_FULL } from '#lib/edits/lookAmount.ts';
import { editor } from '#lib/stores/editor.svelte.ts';
import type { AppliedPreset } from '#lib/stores/editor/presetApplied.svelte.ts';
import { presets } from '#lib/stores/presets.svelte.ts';
import { textMatches } from '#lib/utils/textMatch.ts';

export const ALL_PRESETS = 'all';
export const FAVOURITE_PRESETS = 'favourites';

interface GroupOption {
  key: string;
  label: string;
}

interface Previewing {
  name: string;
  held: boolean;
}

function groupKey(group: string): string {
  return `:${group}`;
}

class PresetStripStore {
  applyOptions = $state<ApplyPresetOptions>(defaultApplyOptions());
  query = $state('');
  private picked = $state(ALL_PRESETS);
  private previewing = $state<Previewing | null>(null);

  readonly groups = $derived(
    presets.grouped
      .map((g) => g.group)
      .sort((a, b) => Number(a === '') - Number(b === '') || a.localeCompare(b))
  );
  readonly hasFavourites = $derived(presets.presets.some((p) => p.favorite));
  readonly groupOptions = $derived<GroupOption[]>([
    ...(this.hasFavourites ? [{ key: FAVOURITE_PRESETS, label: 'Favourites' }] : []),
    { key: ALL_PRESETS, label: 'All' },
    ...this.groups.map((g) => ({ key: groupKey(g), label: g || 'Ungrouped' }))
  ]);
  readonly filterable = $derived(this.groupOptions.length > 2);
  readonly group = $derived(
    this.groupOptions.some((o) => o.key === this.picked) ? this.picked : ALL_PRESETS
  );
  readonly searching = $derived(this.query.trim() !== '');
  readonly shown = $derived.by(() => {
    if (this.searching) return presets.presets.filter((p) => textMatches(p.name, this.query));
    if (this.group === FAVOURITE_PRESETS) return presets.presets.filter((p) => p.favorite);
    if (this.group === ALL_PRESETS) return presets.presets;
    return presets.presets.filter((p) => groupKey(p.group_name ?? '') === this.group);
  });

  get appliedId(): string | null {
    return editor.presetApplied.current?.id ?? null;
  }

  get live(): AppliedPreset | null {
    return editor.presetApplied.current;
  }

  get amount(): number {
    return this.live?.amount ?? this.applyOptions.amount;
  }

  get pill(): PillText | null {
    const previewing = this.previewing;
    if (!previewing || editor.editsPreview?.owner !== 'preset') return null;
    return {
      lead: 'Previewing',
      label: previewing.name,
      tail: previewing.held ? '· release to cancel' : '',
      mono: false
    };
  }

  get appliedPill(): PillText | null {
    const live = this.live;
    if (!live) return null;
    return {
      lead: `${live.name} applied · Amount`,
      label: `${live.amount}%`,
      tail: 'stays live until your next edit',
      mono: true
    };
  }

  ensureLoaded = (): void => {
    if (!presets.loaded && !presets.loading) void presets.load();
  };

  pick = (key: string): void => {
    this.picked = key;
    this.query = '';
  };

  setAmount = (amount: number): void => {
    if (this.live) editor.presetApplied.blend(amount);
    else this.applyOptions.amount = amount;
  };

  commitAmount = (): void => {
    void editor.presetApplied.commitBlend();
  };

  preview = (preset: Preset, held = false): void => {
    this.previewing = { name: preset.name, held };
    editor.previewPreset({ manifest: preset.manifest, opts: this.options() });
  };

  endPreview = (): void => {
    this.previewing = null;
    editor.endPresetPreview();
  };

  apply = (preset: Preset): void => {
    this.previewing = null;
    const opts = this.options();
    this.applyOptions.amount = LOOK_AMOUNT_FULL;
    void editor.applyPreset(preset, opts);
  };

  private options(): ApplyPresetOptions {
    return { ...$state.snapshot(this.applyOptions), amount: this.amount };
  }
}

export const presetStrip = new PresetStripStore();
