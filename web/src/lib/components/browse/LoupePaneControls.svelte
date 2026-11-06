<script lang="ts">
  import { hint } from '#lib/shortcuts/labels.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';
  import { Icon, IconButton } from '@immich/ui';
  import {
    mdiArrowCollapseLeft,
    mdiCheckboxMarkedCircleOutline,
    mdiClose,
    mdiCompare,
    mdiLink,
    mdiLinkOff,
    mdiViewGridOutline
  } from '@mdi/js';

  let { canDrop, onDrop }: { canDrop: boolean; onDrop: () => void } = $props();

  const button = 'touch:size-11';
</script>

<div
  class="flex h-6 items-center gap-1.5 rounded bg-ghost px-2.5 text-[10px] text-light-950/65 touch:h-9 touch:text-xs"
>
  <Icon
    icon={compare.mode === 'compare' ? mdiCompare : mdiViewGridOutline}
    size="13px"
    class="text-primary"
    aria-hidden="true"
  />
  <span class="font-medium text-light-950/85">
    {compare.mode === 'compare' ? 'Compare' : 'Survey'}
  </span>
</div>
<IconButton
  size="tiny"
  variant="ghost"
  color={compare.syncView ? 'primary' : 'secondary'}
  class={button}
  icon={compare.syncView ? mdiLink : mdiLinkOff}
  title={hint('Sync zoom and pan', 'paneSync')}
  aria-label="Sync zoom and pan"
  aria-pressed={compare.syncView}
  onclick={() => (compare.syncView = !compare.syncView)}
/>
{#if compare.mode === 'compare'}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    class={button}
    icon={mdiArrowCollapseLeft}
    title={hint('Make select', 'panePromote')}
    aria-label="Make select"
    disabled={compare.focusIndex === 0}
    onclick={() => compare.promote(compare.focusIndex)}
  />
{:else}
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    class={button}
    icon={mdiCheckboxMarkedCircleOutline}
    title={hint('Keep only this photo', 'surveyKeep')}
    aria-label="Keep only this photo"
    onclick={() => compare.keepOnly(compare.focusIndex)}
  />
{/if}
<IconButton
  size="tiny"
  variant="ghost"
  color="secondary"
  class={button}
  icon={mdiClose}
  title={hint('Drop this photo', 'paneDrop')}
  aria-label="Drop this photo"
  disabled={!canDrop}
  onclick={onDrop}
/>
