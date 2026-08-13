<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent, MaskComponentMode } from '$lib/types/edits';
  import { kindIcon, MODES } from './maskTools';
  import {
    mdiAutoFix,
    mdiChevronDown,
    mdiChevronUp,
    mdiClose,
    mdiEye,
    mdiEyeOff,
    mdiInvertColors
  } from '@mdi/js';

  let {
    layerId,
    comp,
    index,
    total,
    label
  }: {
    layerId: string;
    comp: MaskComponent;
    index: number;
    total: number;
    label: string;
  } = $props();

  const isActive = $derived(editor.activeMaskComponentId === comp.id);

  function patch(patchValue: Partial<MaskComponent>): void {
    editor.patchMaskComponent(layerId, comp.id, patchValue, false);
    void editor.commitMasks();
  }

  function setMode(mode: MaskComponentMode): void {
    patch({ mode });
  }
</script>

<div
  class="flex items-center gap-1.5 px-1 py-0.5 rounded transition-colors cursor-pointer {isActive
    ? 'bg-white/10'
    : 'hover:bg-white/5'}"
  role="button"
  tabindex="0"
  onclick={() => editor.setActiveMaskComponent(isActive ? null : comp.id)}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ')
      editor.setActiveMaskComponent(isActive ? null : comp.id);
  }}
>
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/50 hover:text-immich-dark-fg"
    title={comp.enabled ? 'Disable shape' : 'Enable shape'}
    aria-label="Toggle shape"
    onclick={(e) => {
      e.stopPropagation();
      patch({ enabled: !comp.enabled });
    }}
  >
    <Icon path={comp.enabled ? mdiEye : mdiEyeOff} size={12} />
  </button>
  <Icon
    path={comp.generated ? mdiAutoFix : kindIcon(comp.kind)}
    size={12}
    class="opacity-50 shrink-0"
  />
  <span class="text-[11px] text-immich-dark-fg/70 truncate flex-1">{label}</span>
  {#if index === 0}
    <span
      class="shrink-0 text-[9px] uppercase tracking-wider text-immich-dark-fg/30"
      title="The first shape starts the mask">Base</span
    >
  {/if}
  <button
    type="button"
    class="shrink-0 text-immich-dark-fg/40 hover:text-red-400 transition-colors"
    title="Delete shape"
    aria-label="Delete shape"
    onclick={(e) => {
      e.stopPropagation();
      void editor.removeMaskComponent(layerId, comp.id);
    }}
  >
    <Icon path={mdiClose} size={12} />
  </button>
</div>
{#if isActive}
  <div class="flex items-center gap-2 px-2 pb-1.5 pt-1">
    {#if index > 0}
      <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
        {#each MODES as m (m.value)}
          <button
            type="button"
            class="px-1.5 h-5 inline-flex items-center transition-colors {comp.mode === m.value
              ? 'bg-white/15 text-immich-dark-fg'
              : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
            title={m.hint}
            aria-label={m.hint}
            onclick={() => setMode(m.value)}
          >
            <Icon path={m.icon} size={12} />
          </button>
        {/each}
      </div>
    {/if}
    <button
      type="button"
      class="shrink-0 inline-flex items-center gap-1 px-1.5 h-5 rounded ring-1 ring-white/10 text-[10px] transition-colors {comp.invert
        ? 'bg-white/15 text-immich-dark-fg'
        : 'text-immich-dark-fg/50 hover:bg-white/10 hover:text-immich-dark-fg'}"
      aria-pressed={comp.invert}
      title={comp.invert
        ? 'Using everything outside this shape. Click to go back.'
        : 'Use everything outside this shape instead'}
      onclick={() => patch({ invert: !comp.invert })}
    >
      <Icon path={mdiInvertColors} size={11} />
      Invert
    </button>
    {#if total > 1}
      <div class="ml-auto flex items-center gap-1">
        {#if index > 0}
          <button
            type="button"
            class="px-1.5 py-0.5 rounded text-[10px] text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
            title="Start the mask from this shape"
            onclick={() => void editor.reorderMaskComponent(layerId, comp.id, 0)}
          >
            Make base
          </button>
        {/if}
        <button
          type="button"
          class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
          title="Apply this shape earlier"
          aria-label="Move shape up"
          disabled={index === 0}
          onclick={() => void editor.reorderMaskComponent(layerId, comp.id, index - 1)}
        >
          <Icon path={mdiChevronUp} size={12} />
        </button>
        <button
          type="button"
          class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
          title="Apply this shape later"
          aria-label="Move shape down"
          disabled={index === total - 1}
          onclick={() => void editor.reorderMaskComponent(layerId, comp.id, index + 1)}
        >
          <Icon path={mdiChevronDown} size={12} />
        </button>
      </div>
    {/if}
  </div>
{/if}
