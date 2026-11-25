<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { placeSections, type PlaceSuggestion } from '#lib/geotag/placeSuggestions.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';

  let { onPick }: { onPick: (suggestion: PlaceSuggestion) => void } = $props();

  let open = $state(false);

  const sections = $derived(
    placeSections({
      query: '',
      direct: null,
      saved: places.saved,
      recent: places.freshRecent,
      found: []
    }).filter((section) => section.items.length > 0)
  );
</script>

<Popover
  {open}
  onOpenChange={(next) => (open = next)}
  align="end"
  title="Saved places"
  contentClass="w-72"
>
  {#snippet trigger(props)}
    <Button
      {...props}
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-7.5 shrink-0 gap-1 rounded-lg bg-light-200 ps-2.5 pe-1.5 font-normal hover:bg-light-300 touch:h-11 touch:px-3"
      aria-label="Saved places"
    >
      <span class="text-label text-light-950 touch:text-body">Saved</span>
      <Icon icon={mdiChevronDown} size="16px" class="text-dark/65" aria-hidden />
    </Button>
  {/snippet}
  {#each sections as section (section.id)}
    <div role="group" aria-label={section.title}>
      {#if section.heading}
        <p class="px-3 pt-2 pb-1 section-label text-dark/65 touch:px-4">{section.heading}</p>
      {/if}
      {#each section.items as suggestion (suggestion.key)}
        <MenuRow
          icon={suggestion.icon ?? undefined}
          class="h-auto min-h-9 py-1.5 touch:min-h-13"
          onclick={() => {
            open = false;
            onPick(suggestion);
          }}
        >
          <span class="flex min-w-0 flex-1 flex-col">
            <span class="truncate">{suggestion.place.name}</span>
            {#if suggestion.place.detail}
              <span class="truncate text-caption text-dark/65">{suggestion.place.detail}</span>
            {/if}
          </span>
        </MenuRow>
      {/each}
    </div>
  {:else}
    <p class="px-3 py-3 text-caption text-dark/65 touch:px-4">
      {places.loaded
        ? 'No saved places yet. Save one with the star by the pin, or on the map.'
        : 'Loading…'}
    </p>
  {/each}
</Popover>
