<script lang="ts">
  import { searchPlaces, type Place } from '#lib/api/map.ts';
  import SearchField from '#lib/components/SearchField.svelte';
  import { errorMessage } from '#lib/utils/errors.ts';

  let { onPick }: { onPick: (place: Place) => void } = $props();

  const DEBOUNCE_MS = 300;

  let query = $state('');
  let results = $state<Place[]>([]);
  let status = $state('');
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: AbortController | null = null;

  function search(text: string): void {
    clearTimeout(timer);
    pending?.abort();
    const trimmed = text.trim();
    if (trimmed.length < 2) {
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

  function pick(place: Place): void {
    onPick(place);
    results = [];
    query = place.name;
  }

  function keydown(event: KeyboardEvent): void {
    const first = results[0];
    if (event.key === 'Enter') {
      event.preventDefault();
      if (first) pick(first);
    }
  }

  $effect(() => () => {
    clearTimeout(timer);
    pending?.abort();
  });
</script>

<div class="relative z-10">
  <SearchField bind:value={query} label="Search for a place" oninput={search} onkeydown={keydown} />
  {#if results.length > 0}
    <ul
      aria-label="Places"
      class="absolute inset-x-0 top-full mt-1 overflow-hidden rounded-lg border border-hairline bg-light-100 py-1 shadow-popover"
    >
      {#each results as place (`${place.lat},${place.lon},${place.name}`)}
        <li>
          <button
            type="button"
            class="flex w-full flex-col items-start px-3 py-2 text-start outline-none hover:bg-subtle focus-visible:bg-subtle touch:py-3"
            onclick={() => pick(place)}
          >
            <span class="text-sm text-light-950 touch:text-body">{place.name}</span>
            {#if place.detail}
              <span class="text-caption text-dark/65 touch:text-sm">{place.detail}</span>
            {/if}
          </button>
        </li>
      {/each}
      <li class="border-t border-hairline px-3 pt-1.5 pb-0.5 text-micro text-dark/65">
        Search by Photon · OpenStreetMap
      </li>
    </ul>
  {:else if status}
    <p class="mt-1 text-caption text-dark/65" role="status">{status}</p>
  {/if}
</div>
