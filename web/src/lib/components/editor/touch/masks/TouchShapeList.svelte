<script lang="ts">
  import { rowHold } from '#lib/gestures/rowHold.svelte.ts';
  import { kindIcon, modeVerb, patchShape } from '#lib/panels/maskTools.ts';
  import { shapeSubtitle } from '#lib/panels/maskShapeText.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskHover } from '#lib/stores/editor/maskHover.svelte.ts';
  import type { MaskComponentMode, MaskLayer } from '#lib/types/edits.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiAutoFix, mdiDotsHorizontal, mdiEye, mdiEyeOff } from '@mdi/js';

  let {
    layer,
    labels,
    onmenu
  }: { layer: MaskLayer; labels: readonly string[]; onmenu: (componentId: string) => void } =
    $props();

  const BADGE_TONES: Record<MaskComponentMode, string> = {
    add: 'bg-success/15 text-success',
    subtract: 'bg-danger/15 text-danger',
    intersect: 'bg-primary/15 text-primary'
  };

  const hold = rowHold({
    attribute: 'data-shape-hold',
    start: (id) => {
      const index = layer.components.findIndex((comp) => comp.id === id);
      const comp = layer.components[index];
      if (!comp) return;
      maskHover.enter({
        layerId: layer.id,
        componentId: comp.id,
        label: labels[index] ?? '',
        mode: index === 0 ? null : comp.mode
      });
    },
    end: maskHover.clear
  });
</script>

<div role="list" aria-label="Shapes" class="flex flex-col" {@attach hold.attachment}>
  {#each layer.components as comp, index (comp.id)}
    {@const active = editor.activeMaskComponentId === comp.id}
    {@const label = labels[index] ?? ''}
    <div
      role="listitem"
      data-shape-hold={comp.id}
      data-testid="mask-shape-row"
      class="flex h-12 items-center gap-1 border-b border-hairline pe-1 {active
        ? 'bg-primary/10'
        : ''} {comp.enabled ? '' : 'opacity-55'}"
    >
      <Button
        type="button"
        variant="ghost"
        color="secondary"
        class="h-full min-w-0 flex-1 justify-start gap-3 rounded-none ps-4 pe-1 font-normal"
        aria-label={label}
        aria-pressed={active}
        onclick={() => editor.setActiveMaskComponent(active ? null : comp.id)}
      >
        <Icon
          icon={comp.generated ? mdiAutoFix : kindIcon(comp.kind)}
          size="18px"
          class="shrink-0 opacity-80"
          aria-hidden="true"
        />
        <span class="flex min-w-0 flex-1 flex-col text-left">
          <span class="truncate text-[14px] text-dark">{label}</span>
          <span class="truncate text-[11px] text-dark/65">{shapeSubtitle(comp)}</span>
        </span>
      </Button>
      {#if index === 0}
        <span
          class="shrink-0 rounded-full border border-hairline px-2 py-0.5 text-[10.5px] tracking-wider text-dark/65 uppercase"
        >
          Base
        </span>
      {:else}
        <span class="shrink-0 rounded-full px-2 py-0.5 text-[11.5px] {BADGE_TONES[comp.mode]}">
          {modeVerb(comp.mode)}
        </span>
      {/if}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="size-9 shrink-0"
        icon={comp.enabled ? mdiEye : mdiEyeOff}
        aria-label={comp.enabled ? `Hide ${label}` : `Show ${label}`}
        title=""
        onclick={() => patchShape(layer.id, comp.id, { enabled: !comp.enabled })}
      />
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="size-9 shrink-0"
        icon={mdiDotsHorizontal}
        aria-label="{label} actions"
        title=""
        onclick={() => onmenu(comp.id)}
      />
    </div>
  {/each}
</div>
