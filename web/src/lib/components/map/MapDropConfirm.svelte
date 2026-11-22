<script lang="ts">
  import { coordinateLabel } from '#lib/browse/info/photoInfo.ts';
  import MapPin from '#lib/components/map/MapPin.svelte';
  import { dropQuestion, placeLine, type DropConfirm } from '#lib/map/dropConfirm.svelte.ts';
  import { Button } from '@immich/ui';
  import type { Map as MapLibreMap } from 'maplibre-gl';

  let { confirm, map }: { confirm: DropConfirm; map: MapLibreMap | null } = $props();

  const CARD_SPACE = 150;
  const CARD_HALF = 136;

  let point = $state<{ x: number; y: number; width: number } | null>(null);

  $effect(() => {
    const drop = confirm.pending;
    const target = map;
    if (!drop || !target) {
      point = null;
      return;
    }
    const place = (): void => {
      const at = target.project([drop.lon, drop.lat]);
      point = { x: at.x, y: at.y, width: target.getContainer().clientWidth };
    };
    place();
    target.on('move', place);
    return () => target.off('move', place);
  });

  $effect(() => {
    if (!confirm.pending) return;
    const key = (event: KeyboardEvent): void => {
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      confirm.cancel();
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });

  const focus = (card: HTMLElement): void => {
    card.querySelector<HTMLButtonElement>('[data-drop-ok]')?.focus();
  };
</script>

{#if confirm.pending && point}
  {@const drop = confirm.pending}
  {@const below = point.y < CARD_SPACE}
  {#if drop.pin}
    <div
      class="pointer-events-none absolute z-20 -translate-x-1/2 -translate-y-full"
      style:left="{point.x}px"
      style:top="{point.y}px"
      aria-hidden="true"
    >
      <MapPin />
    </div>
  {/if}
  <div
    role="alertdialog"
    aria-label={dropQuestion(drop)}
    class="absolute z-30 w-64 -translate-x-1/2 rounded-xl bg-light-100 p-3 shadow-popover ring-1 ring-hairline"
    class:-translate-y-full={!below}
    style:left="{Math.min(Math.max(CARD_HALF, point.x), point.width - CARD_HALF)}px"
    style:top="{below ? point.y + 16 : point.y - 56}px"
    data-drop-confirm
    {@attach focus}
  >
    <p class="text-sm font-semibold text-light-950">{dropQuestion(drop)}</p>
    <p class="mt-0.5 truncate text-caption text-dark/65">
      {drop.named
        ? (placeLine(drop.place) ?? coordinateLabel(drop.lat, drop.lon))
        : coordinateLabel(drop.lat, drop.lon)}
    </p>
    <div class="mt-3 flex justify-end gap-2">
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        onclick={confirm.cancel}
      >
        Cancel
      </Button>
      <Button size="small" class="touch:h-11" data-drop-ok onclick={confirm.confirm}>
        {drop.verb === 'move' ? 'Move' : 'Place'}
      </Button>
    </div>
  </div>
{/if}
