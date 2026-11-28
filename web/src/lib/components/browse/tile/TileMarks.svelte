<script lang="ts">
  import type { TopMark } from '#lib/browse/tileContent.ts';
  import { OVERLAY_ICON_PATHS } from '#lib/browse/overlayIcons.ts';
  import { LABEL_NAMES, LABEL_TEXT, type LabelColor } from '#lib/stores/labels.ts';
  import { Icon } from '@immich/ui';

  let {
    marks,
    label,
    low,
    class: className = ''
  }: {
    marks: readonly TopMark[];
    label: LabelColor | null;
    low: boolean;
    class?: string;
  } = $props();
</script>

<div
  class="pointer-events-none absolute top-1 right-1 flex items-center gap-1 text-white drop-shadow-md transition-opacity {low
    ? 'h-5 max-w-[calc(100%-2.125rem)] flex-row-reverse flex-wrap content-start overflow-hidden'
    : 'flex-col'} {className}"
>
  {#each marks as mark (mark)}
    {#if mark === 'edited'}
      <div
        role="img"
        aria-label="Edited"
        class="grid size-5 shrink-0 place-items-center rounded-full bg-black/55"
      >
        <Icon icon={OVERLAY_ICON_PATHS.edited} size="12px" aria-hidden="true" />
      </div>
    {:else if mark === 'label' && label}
      <div
        role="img"
        aria-label="{LABEL_NAMES[label]} label"
        class="grid size-5 shrink-0 place-items-center {LABEL_TEXT[label]}"
      >
        <Icon icon={OVERLAY_ICON_PATHS.label} size="14px" aria-hidden="true" />
      </div>
    {:else if mark !== 'label'}
      <div
        role="img"
        aria-label={mark === 'favorite' ? 'Favorite' : 'Rejected'}
        class="grid size-5 shrink-0 place-items-center"
      >
        <Icon icon={OVERLAY_ICON_PATHS[mark]} size="16px" aria-hidden="true" />
      </div>
    {/if}
  {/each}
</div>
