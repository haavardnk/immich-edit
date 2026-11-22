<script lang="ts">
  import MapPin from '#lib/components/map/MapPin.svelte';
  import type { PhotoDrag } from '#lib/map/photoDrag.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiMapMarkerDown } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    drag,
    onDrop,
    class: className = '',
    children
  }: {
    drag: PhotoDrag;
    onDrop: (ids: readonly string[], client: { x: number; y: number }) => void;
    class?: string;
    children: Snippet;
  } = $props();

  let zone: HTMLDivElement | undefined = $state();

  const point = $derived.by(() => {
    const client = drag.client;
    if (!client || !zone || !drag.over) return null;
    const box = zone.getBoundingClientRect();
    return { x: client.x - box.left, y: client.y - box.top };
  });

  function over(event: DragEvent): void {
    if (!drag.ids) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'move';
    drag.over = true;
    drag.client = { x: event.clientX, y: event.clientY };
  }

  function leave(event: DragEvent): void {
    const box = zone?.getBoundingClientRect();
    const inside =
      box !== undefined &&
      event.clientX > box.left &&
      event.clientX < box.right &&
      event.clientY > box.top &&
      event.clientY < box.bottom;
    if (inside) return;
    drag.over = false;
  }

  function drop(event: DragEvent): void {
    const ids = drag.ids;
    if (!ids) return;
    event.preventDefault();
    onDrop(ids, { x: event.clientX, y: event.clientY });
    drag.end();
  }
</script>

<div
  bind:this={zone}
  class="relative {className}"
  role="region"
  aria-label="Drop photos on the map"
  ondragover={over}
  ondragenter={over}
  ondragleave={leave}
  ondrop={drop}
>
  {@render children()}
  {#if drag.ids && drag.over}
    <div
      class="pointer-events-none absolute inset-0 z-10 ring-3 ring-primary ring-inset"
      aria-hidden="true"
    ></div>
    <p
      class="pointer-events-none absolute top-3 left-1/2 z-10 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-primary px-3 py-1.5 text-sm font-medium text-light shadow-lg"
      role="status"
    >
      <Icon icon={mdiMapMarkerDown} size="16px" aria-hidden="true" />
      Drop to place {countOf(drag.ids.length, 'photo')}
    </p>
    {#if point}
      <div
        class="pointer-events-none absolute z-10 -translate-x-1/2 -translate-y-full"
        style:left="{point.x}px"
        style:top="{point.y}px"
        data-drop-pin
        aria-hidden="true"
      >
        <MapPin />
      </div>
    {/if}
  {/if}
</div>
