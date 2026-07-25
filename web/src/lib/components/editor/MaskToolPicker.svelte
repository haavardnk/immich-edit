<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import {
    mdiGradientHorizontal,
    mdiCircleOutline,
    mdiBrush,
    mdiBrightness6,
    mdiPalette,
    mdiAutoFix,
    mdiCursorDefaultClick,
    mdiDownloadOutline
  } from '@mdi/js';
  import type { MaskKind } from '$lib/api/masks';
  import type { ManualTool } from '$lib/types/masks';

  let {
    aiKinds,
    busy = false,
    onManual,
    onAi
  }: {
    aiKinds: { kind: MaskKind; installed: boolean }[];
    busy?: boolean;
    onManual: (tool: ManualTool) => void;
    onAi: (kind: MaskKind, installed: boolean) => void;
  } = $props();

  const MANUAL: { tool: ManualTool; label: string; hint: string; icon: string }[] = [
    {
      tool: 'linear',
      label: 'Linear gradient',
      hint: 'Fades across a straight edge',
      icon: mdiGradientHorizontal
    },
    {
      tool: 'radial',
      label: 'Radial gradient',
      hint: 'Fades out from an ellipse',
      icon: mdiCircleOutline
    },
    { tool: 'brush', label: 'Brush', hint: 'Paint the area by hand', icon: mdiBrush },
    {
      tool: 'luma_range',
      label: 'Luminance range',
      hint: 'Picks a brightness band',
      icon: mdiBrightness6
    },
    {
      tool: 'color_range',
      label: 'Color range',
      hint: 'Picks colors near a sample',
      icon: mdiPalette
    }
  ];

  const AI_HINTS: Record<string, string> = {
    subject: 'Isolates the main subject',
    people: 'Finds every person',
    sky: 'Finds the sky',
    depth: 'Selects a distance band',
    click: 'Click the photo to select'
  };

  function aiLabel(kind: string): string {
    if (kind === 'subject') return 'Subject';
    if (kind === 'people') return 'People';
    if (kind === 'sky') return 'Sky';
    if (kind === 'depth') return 'Depth';
    if (kind === 'click') return 'Click to select';
    return kind;
  }
</script>

<div class="w-56 py-1">
  {#if aiKinds.length > 0}
    <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
      AI
    </div>
    {#each aiKinds as entry (entry.kind)}
      <button
        type="button"
        class="flex items-start gap-2.5 w-full px-3 py-1.5 text-left transition-colors hover:bg-white/10 disabled:opacity-40 disabled:cursor-not-allowed {entry.installed
          ? ''
          : 'text-immich-dark-fg/40'}"
        disabled={busy && entry.installed && entry.kind !== 'click'}
        aria-label={aiLabel(entry.kind)}
        onclick={() => onAi(entry.kind, entry.installed)}
      >
        <Icon
          path={entry.kind === 'click' ? mdiCursorDefaultClick : mdiAutoFix}
          size={14}
          class="mt-0.5 shrink-0 opacity-70"
        />
        <span class="flex-1 min-w-0">
          <span class="block text-xs truncate">{aiLabel(entry.kind)}</span>
          <span class="block text-[10px] text-immich-dark-fg/40 truncate">
            {entry.installed ? AI_HINTS[entry.kind] : 'Needs a model download'}
          </span>
        </span>
        {#if !entry.installed}
          <Icon path={mdiDownloadOutline} size={13} class="mt-0.5 shrink-0" />
        {/if}
      </button>
    {/each}
    <div class="my-1 border-t border-white/10"></div>
  {/if}
  <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
    Manual
  </div>
  {#each MANUAL as item (item.tool)}
    <button
      type="button"
      class="flex items-start gap-2.5 w-full px-3 py-1.5 text-left transition-colors hover:bg-white/10"
      aria-label={item.label}
      onclick={() => onManual(item.tool)}
    >
      <Icon path={item.icon} size={14} class="mt-0.5 shrink-0 opacity-70" />
      <span class="flex-1 min-w-0">
        <span class="block text-xs truncate">{item.label}</span>
        <span class="block text-[10px] text-immich-dark-fg/40 truncate">{item.hint}</span>
      </span>
    </button>
  {/each}
</div>
