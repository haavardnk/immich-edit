<script lang="ts">
  import ChipChoice from '#lib/components/chips/ChipChoice.svelte';
  import { chipFill } from '#lib/components/chips/chipStyles.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import type { CropRatio } from '#lib/api/preferences.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { fitCount } from '#lib/utils/fitCount.ts';
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

  const KINDS = [
    { value: 'original', label: 'Original' },
    { value: 'free', label: 'Free' }
  ];

  let moreOpen = $state(false);
  let available = $state(0);
  let gap = $state(0);
  let widths = $state<number[]>([]);
  let overflow = $state(0);

  const portrait = $derived(draftPortrait());
  const ratios = $derived(
    [...CHIP_RATIOS, ...MORE_RATIOS, ...preferences.savedRatios].filter(
      (ratio, index, all) => all.findIndex((other) => ratioKey(other) === ratioKey(ratio)) === index
    )
  );
  const options = $derived([
    ...KINDS,
    ...ratios.map((ratio) => ({ value: ratioKey(ratio), label: ratioLabel(ratio, portrait) }))
  ]);
  const shown = $derived(
    available > 0 ? fitCount(widths, available, gap, overflow) : KINDS.length + CHIP_RATIOS.length
  );
  const folded = $derived(ratios.slice(Math.max(0, shown - KINDS.length)));
  const moreActive = $derived(folded.some((ratio) => isPicked(ratio)));

  function pick(key: string): void {
    if (key === 'original' || key === 'free') {
      pickKind(key);
      return;
    }
    const ratio = ratios.find((candidate) => ratioKey(candidate) === key);
    if (ratio) pickRatio(ratio, portrait);
  }

  function measure(row: HTMLElement): () => void {
    const update = (): void => {
      const ruler = row.querySelector<HTMLElement>('[data-chip-ruler]');
      if (!ruler) return;
      const sizes = Array.from(ruler.children, (child) => child.getBoundingClientRect().width);
      gap = Number.parseFloat(getComputedStyle(ruler).columnGap) || 0;
      overflow = Math.ceil(sizes.pop() ?? 0);
      widths = sizes.map((size) => Math.ceil(size));
      available = Math.floor(row.clientWidth);
    };
    const observer = new ResizeObserver(update);
    observer.observe(row);
    const ruler = row.querySelector('[data-chip-ruler]');
    if (ruler) observer.observe(ruler);
    update();
    return () => observer.disconnect();
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

<div class="relative min-w-0" {@attach measure}>
  <ChipChoice
    label="Aspect ratio"
    fill
    options={options.slice(0, shown)}
    value={aspectKey(draftAspect())}
    onChange={pick}
  >
    {#if folded.length > 0}
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
            class={chipFill}
            aria-label="More ratios"
            aria-pressed={moreActive}
          >
            <Icon icon={mdiDotsHorizontal} size="1.15em" aria-hidden="true" />
          </Button>
        {/snippet}
        {#each folded as ratio (ratioKey(ratio))}
          {@render menuRatio(ratio)}
        {/each}
      </Popover>
    {/if}
  </ChipChoice>
  <div
    data-chip-ruler
    aria-hidden="true"
    inert
    class="pointer-events-none invisible absolute top-0 left-0 flex w-max gap-1 touch:gap-1.5"
  >
    {#each options as option (option.value)}
      <span class={chipFill}>{option.label}</span>
    {/each}
    <span class={chipFill}><Icon icon={mdiDotsHorizontal} size="1.15em" /></span>
  </div>
</div>
