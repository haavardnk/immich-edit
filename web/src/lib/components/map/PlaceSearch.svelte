<script lang="ts">
  import { searchPlaces, type Place } from '#lib/api/map.ts';
  import SearchField from '#lib/components/SearchField.svelte';
  import { parseCoordinates } from '#lib/geotag/coordinates.ts';
  import { placeSections, type PlaceSuggestion } from '#lib/geotag/placeSuggestions.ts';
  import { places } from '#lib/stores/places.svelte.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { Icon } from '@immich/ui';
  import { onMount } from 'svelte';

  let { onPick, photon }: { onPick: (place: Place) => void; photon: boolean } = $props();

  const DEBOUNCE_MS = 300;

  let query = $state('');
  let results = $state<Place[]>([]);
  let status = $state('');
  let focused = $state(false);
  let picked = $state(false);
  let box = $state<HTMLElement | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: AbortController | null = null;

  const sections = $derived(
    placeSections({
      query,
      direct: parseCoordinates(query),
      saved: places.saved,
      recent: places.freshRecent,
      found: results
    })
  );
  const first = $derived(sections.flatMap((section) => section.items)[0] ?? null);
  const shown = $derived(!picked && (focused || results.length > 0) && first !== null);

  function search(text: string): void {
    picked = false;
    clearTimeout(timer);
    pending?.abort();
    const trimmed = text.trim();
    if (trimmed.length < 2 || !photon || parseCoordinates(trimmed)) {
      results = [];
      status = '';
      return;
    }
    timer = setTimeout(() => {
      const controller = new AbortController();
      pending = controller;
      searchPlaces(trimmed, controller.signal)
        .then((found) => {
          if (controller.signal.aborted) return;
          results = found;
          status = found.length === 0 ? 'No places found' : '';
        })
        .catch((error: unknown) => {
          if (controller.signal.aborted) return;
          results = [];
          status = errorMessage(error);
        });
    }, DEBOUNCE_MS);
  }

  function pick(suggestion: PlaceSuggestion): void {
    onPick(suggestion.place);
    results = [];
    picked = true;
    query = suggestion.place.name;
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    if (first) pick(first);
  }

  function focusOut(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (!(next instanceof Node) || !box?.contains(next)) focused = false;
  }

  onMount(() => {
    void places.ensure();
  });

  $effect(() => () => {
    clearTimeout(timer);
    pending?.abort();
  });
</script>

<div
  bind:this={box}
  class="relative z-10"
  onfocusin={() => {
    focused = true;
    picked = false;
  }}
  onfocusout={focusOut}
>
  <SearchField
    bind:value={query}
    label="Search for a place"
    placeholder={photon
      ? 'Search places, or paste coordinates or a map link'
      : 'Paste coordinates or a map link'}
    oninput={search}
    onkeydown={keydown}
  />
  {#if shown}
    <div
      class="absolute inset-x-0 top-full mt-1 max-h-80 overflow-y-auto rounded-lg border border-hairline bg-light-100 py-1 shadow-popover touch:max-h-[45vh]"
    >
      {#each sections as section (section.id)}
        {#if section.items.length > 0}
          <ul aria-label={section.title}>
            {#if section.heading}
              <li class="px-3 pt-2 pb-1 touch:px-4">
                <span class="section-label text-dark/65">{section.heading}</span>
              </li>
            {/if}
            {#each section.items as suggestion (suggestion.key)}
              <li>
                <button
                  type="button"
                  class="flex w-full items-center gap-3 px-3 py-2 text-start outline-none hover:bg-subtle focus-visible:bg-subtle touch:px-4 touch:py-3"
                  onclick={() => pick(suggestion)}
                >
                  {#if suggestion.icon}
                    <Icon
                      icon={suggestion.icon}
                      size="16px"
                      class="shrink-0 {suggestion.saved ? 'text-primary' : 'text-dark/65'}"
                      aria-hidden
                    />
                  {/if}
                  <span class="flex min-w-0 flex-1 flex-col">
                    <span class="truncate text-sm text-light-950 touch:text-body">
                      {suggestion.place.name}
                    </span>
                    {#if suggestion.place.detail}
                      <span class="truncate text-caption text-dark/65 touch:text-sm">
                        {suggestion.place.detail}
                      </span>
                    {/if}
                  </span>
                  {#if suggestion.trail}
                    <span class="shrink-0 text-caption text-dark/65">{suggestion.trail}</span>
                  {/if}
                </button>
              </li>
            {/each}
            {#if section.id === 'found'}
              <li class="border-t border-hairline px-3 pt-1.5 pb-0.5 text-micro text-dark/65">
                Search by Photon · OpenStreetMap
              </li>
            {/if}
          </ul>
        {/if}
      {/each}
    </div>
  {:else if status && !picked}
    <p class="mt-1 text-caption text-dark/65" role="status">{status}</p>
  {/if}
</div>
