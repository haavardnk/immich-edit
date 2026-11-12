<script lang="ts">
  import ChipChoice from '#lib/components/chips/ChipChoice.svelte';
  import { chipTone } from '#lib/components/chips/chipStyles.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import type { CropRatio } from '#lib/api/preferences.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { mdiCheck, mdiDotsHorizontal } from '@mdi/js';
  import { Button, Icon } from '@immich/ui';
  import {
    aspectKey,
    CHIP_RATIOS,
    MORE_RATIOS,
    ratioKey,
    ratioLabel
  } from '#lib/panels/geometry/aspects.ts';
  import {
    draftAspect,
    draftPortrait,
    isPicked,
    pickKind,
    pickRatio
  } from '#lib/panels/geometry/ratioActions.ts';

  let moreOpen = $state(false);

  const portrait = $derived(draftPortrait());
  const options = $derived([
    { value: 'original', label: 'Original' },
    { value: 'free', label: 'Free' },
    ...CHIP_RATIOS.map((ratio) => ({
      value: ratioKey(ratio),
      label: ratioLabel(ratio, portrait)
    }))
  ]);
  const moreActive = $derived(
    [...MORE_RATIOS, ...preferences.savedRatios].some((ratio) => isPicked(ratio))
  );

  function pick(key: string): void {
    if (key === 'original' || key === 'free') {
      pickKind(key);
      return;
    }
    const ratio = CHIP_RATIOS.find((candidate) => ratioKey(candidate) === key);
    if (ratio) pickRatio(ratio, portrait);
  }
</script>

{#snippet menuRatio(ratio: CropRatio)}
  {@const picked = isPicked(ratio)}
  <MenuRow
    label={ratioLabel(ratio, portrait)}
    pressed={picked}
    onclick={() => {
      moreOpen = false;
      pickRatio(ratio, portrait);
    }}
  >
    {#if picked}
      <Icon icon={mdiCheck} size="14px" class="shrink-0 text-primary" aria-hidden="true" />
    {/if}
  </MenuRow>
{/snippet}

<ChipChoice
  label="Aspect ratio"
  tone="dense"
  {options}
  value={aspectKey(draftAspect())}
  onChange={pick}
>
  <Popover
    open={moreOpen}
    align="end"
    onOpenChange={(next) => (moreOpen = next)}
    title="More ratios"
    tooltip="More ratios"
  >
    {#snippet trigger(props)}
      <Button
        {...props}
        size="tiny"
        variant="ghost"
        color="secondary"
        class={chipTone.dense}
        aria-label="More ratios"
        aria-pressed={moreActive}
      >
        <Icon icon={mdiDotsHorizontal} size="1.15em" aria-hidden="true" />
      </Button>
    {/snippet}
    {#each MORE_RATIOS as ratio (ratioKey(ratio))}
      {@render menuRatio(ratio)}
    {/each}
    {#if preferences.savedRatios.length}
      <div role="separator" class="my-1 border-t border-hairline"></div>
      {#each preferences.savedRatios as ratio (ratioKey(ratio))}
        {@render menuRatio(ratio)}
      {/each}
    {/if}
  </Popover>
</ChipChoice>
