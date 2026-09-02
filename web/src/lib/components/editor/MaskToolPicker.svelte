<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import {
    mdiGradientHorizontal,
    mdiCircleOutline,
    mdiBrush,
    mdiBrightness6,
    mdiPalette,
    mdiAutoFix,
    mdiCursorDefaultClick,
    mdiSelectDrag,
    mdiVectorDifference,
    mdiVectorPolygon,
    mdiChevronDown,
    mdiChevronUp,
    mdiDownloadOutline
  } from '@mdi/js';
  import type { MaskKind, SemanticClass } from '$lib/api/masks';
  import { generatedLabel, visibleSceneClasses, type ManualTool } from '$lib/types/masks';

  let {
    aiKinds,
    semanticClasses = [],
    aiUnavailable = null,
    busy = false,
    onManual,
    onAi,
    onBox,
    onBackground
  }: {
    aiKinds: { kind: MaskKind; installed: boolean }[];
    semanticClasses?: SemanticClass[];
    aiUnavailable?: string | null;
    busy?: boolean;
    onManual: (tool: ManualTool) => void;
    onAi: (kind: MaskKind, installed: boolean, maskClass?: string) => void;
    onBox: () => void;
    onBackground: () => void;
  } = $props();

  let semanticOpen = $state(false);

  const sceneClasses = $derived(visibleSceneClasses(semanticClasses, aiKinds));

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
      tool: 'polygon',
      label: 'Polygon',
      hint: 'Drag the corners to fit a shape',
      icon: mdiVectorPolygon
    },
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
    semantic: 'Water, foliage, buildings and more',
    click: 'Click the photo to select'
  };
</script>

<div class="w-56 py-1">
  {#if aiKinds.length > 0}
    <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">AI</div>
    {#each aiKinds as entry (entry.kind)}
      <Button
        type="button"
        size="tiny"
        variant="ghost"
        color="secondary"
        class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left {entry.installed
          ? ''
          : 'text-dark/65'}"
        disabled={busy && entry.installed && entry.kind !== 'click'}
        aria-label={generatedLabel(entry.kind)}
        onclick={() => {
          if (entry.kind === 'semantic' && entry.installed) {
            semanticOpen = !semanticOpen;
            return;
          }
          onAi(entry.kind, entry.installed);
        }}
      >
        <Icon
          icon={entry.kind === 'click' ? mdiCursorDefaultClick : mdiAutoFix}
          size="14px"
          class="mt-0.5 shrink-0 opacity-70"
          aria-hidden="true"
        />
        <span class="flex-1 min-w-0">
          <span class="block text-xs truncate">{generatedLabel(entry.kind)}</span>
          <span class="block text-[10px] text-dark/65 truncate">
            {entry.installed ? AI_HINTS[entry.kind] : 'Needs a model download'}
          </span>
        </span>
        {#if !entry.installed}
          <Icon icon={mdiDownloadOutline} size="13px" class="mt-0.5 shrink-0" aria-hidden="true" />
        {:else if entry.kind === 'semantic'}
          <Icon
            icon={semanticOpen ? mdiChevronUp : mdiChevronDown}
            size="13px"
            class="mt-0.5 shrink-0"
            aria-hidden="true"
          />
        {/if}
      </Button>
      {#if entry.kind === 'subject' && entry.installed}
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color="secondary"
          class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left"
          aria-label="Background"
          disabled={busy}
          onclick={onBackground}
        >
          <Icon
            icon={mdiVectorDifference}
            size="14px"
            class="mt-0.5 shrink-0 opacity-70"
            aria-hidden="true"
          />
          <span class="flex-1 min-w-0">
            <span class="block text-xs truncate">Background</span>
            <span class="block text-[10px] text-dark/65 truncate">
              Everything but the subject
            </span>
          </span>
        </Button>
      {/if}
      {#if entry.kind === 'click' && entry.installed}
        <Button
          type="button"
          size="tiny"
          variant="ghost"
          color="secondary"
          class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left"
          aria-label="Box select"
          onclick={onBox}
        >
          <Icon
            icon={mdiSelectDrag}
            size="14px"
            class="mt-0.5 shrink-0 opacity-70"
            aria-hidden="true"
          />
          <span class="flex-1 min-w-0">
            <span class="block text-xs truncate">Box select</span>
            <span class="block text-[10px] text-dark/65 truncate">
              Drag a box around a subject
            </span>
          </span>
        </Button>
      {/if}
      {#if entry.kind === 'semantic' && entry.installed && semanticOpen}
        {#each sceneClasses as cls (cls.id)}
          <Button
            type="button"
            size="tiny"
            variant="ghost"
            color="secondary"
            class="w-full justify-start gap-2.5 rounded-none py-1 pl-10 pr-3 text-left text-xs"
            disabled={busy}
            onclick={() => onAi('semantic', true, cls.id)}
          >
            {cls.name}
          </Button>
        {/each}
      {/if}
    {/each}
    <div class="my-1 border-t border-dark/10"></div>
  {:else if aiUnavailable}
    <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">AI</div>
    <div class="px-3 pb-2 text-[10px] text-dark/65 leading-snug">
      {aiUnavailable}
    </div>
    <div class="my-1 border-t border-dark/10"></div>
  {/if}
  <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">Manual</div>
  {#each MANUAL as item (item.tool)}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left"
      aria-label={item.label}
      onclick={() => onManual(item.tool)}
    >
      <Icon icon={item.icon} size="14px" class="mt-0.5 shrink-0 opacity-70" aria-hidden="true" />
      <span class="flex-1 min-w-0">
        <span class="block text-xs truncate">{item.label}</span>
        <span class="block text-[10px] text-dark/65 truncate">{item.hint}</span>
      </span>
    </Button>
  {/each}
</div>
