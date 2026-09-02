<script lang="ts">
  import { compactSegmentedControlClass } from '$lib/components/editor/controls/segmentedControl';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent, MaskComponentMode } from '$lib/types/edits';
  import { kindIcon, MODES } from './maskTools';
  import { Button, Icon, IconButton } from '@immich/ui';
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
  class="flex h-7 items-center gap-1 rounded-md border px-1 transition-colors {isActive
    ? 'border-primary/20 bg-primary/8'
    : 'border-transparent hover:border-hairline hover:bg-white/4'}"
>
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={comp.enabled ? mdiEye : mdiEyeOff}
    title={comp.enabled ? 'Disable shape' : 'Enable shape'}
    aria-label="Toggle shape"
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      patch({ enabled: !comp.enabled });
    }}
  />
  <Icon
    icon={comp.generated ? mdiAutoFix : kindIcon(comp.kind)}
    size="12px"
    class="opacity-50 shrink-0"
    aria-hidden="true"
  />
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-5 min-w-0 flex-1 justify-start truncate rounded-sm px-1 py-0 text-left text-[11px] text-dark/65 hover:bg-transparent"
    aria-pressed={isActive}
    onclick={() => editor.setActiveMaskComponent(isActive ? null : comp.id)}
  >
    {label}
  </Button>
  {#if index === 0}
    <span
      class="shrink-0 text-[9px] uppercase tracking-wider text-dark/65"
      title="The first shape starts the mask">Base</span
    >
  {/if}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    icon={mdiClose}
    title="Delete shape"
    aria-label="Delete shape"
    onclick={(e: MouseEvent) => {
      e.stopPropagation();
      void editor.removeMaskComponent(layerId, comp.id);
    }}
  />
</div>
{#if isActive}
  <div class="mx-1 flex min-h-7 items-center gap-1 border-l border-primary/25 px-2 pb-1">
    {#if index > 0}
      <div class="{compactSegmentedControlClass} flex text-[10px]">
        {#each MODES as m (m.value)}
          <IconButton
            size="tiny"
            variant="ghost"
            color={comp.mode === m.value ? 'primary' : 'secondary'}
            icon={m.icon}
            title={m.hint}
            aria-label={m.hint}
            aria-pressed={comp.mode === m.value}
            onclick={() => setMode(m.value)}
          />
        {/each}
      </div>
    {/if}
    <Button
      type="button"
      size="tiny"
      variant={comp.invert ? 'filled' : 'ghost'}
      color={comp.invert ? 'primary' : 'secondary'}
      class="shrink-0"
      leadingIcon={mdiInvertColors}
      title={comp.invert
        ? 'Using everything outside this shape. Click to go back.'
        : 'Use everything outside this shape instead'}
      aria-pressed={comp.invert}
      onclick={() => patch({ invert: !comp.invert })}
    >
      Invert
    </Button>
    {#if total > 1}
      <div class="ml-auto flex items-center gap-0.5">
        {#if index > 0}
          <Button
            type="button"
            size="tiny"
            variant="ghost"
            color="secondary"
            title="Start the mask from this shape"
            onclick={() => void editor.reorderMaskComponent(layerId, comp.id, 0)}
          >
            Make base
          </Button>
        {/if}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiChevronUp}
          title="Apply this shape earlier"
          aria-label="Move shape up"
          disabled={index === 0}
          onclick={() => void editor.reorderMaskComponent(layerId, comp.id, index - 1)}
        />
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiChevronDown}
          title="Apply this shape later"
          aria-label="Move shape down"
          disabled={index === total - 1}
          onclick={() => void editor.reorderMaskComponent(layerId, comp.id, index + 1)}
        />
      </div>
    {/if}
  </div>
{/if}
