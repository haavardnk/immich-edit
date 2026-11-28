<script lang="ts">
  import DayHeader from '#lib/components/browse/DayHeader.svelte';
  import { dayTitle, daySubtitle, type DayGroup } from '#lib/browse/dayGroups.ts';
  import { visibleHeaders, type DayHeaderBox } from '#lib/browse/dayLayout.ts';
  import { pickDays } from '#lib/browse/selectDay.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { selection } from '#lib/stores/selection.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';

  let {
    headers,
    days,
    items,
    startIndex,
    endIndex,
    pad,
    locked
  }: {
    headers: readonly DayHeaderBox[];
    days: readonly DayGroup[];
    items: readonly AssetSummary[];
    startIndex: number;
    endIndex: number;
    pad: number;
    locked: boolean;
  } = $props();

  let picking = $state<string | null>(null);
  let anchor: string | null = null;

  $effect(() => {
    if (!selection.active) anchor = null;
  });

  const shown = $derived.by(() => {
    const today = new Date();
    return visibleHeaders(headers, startIndex, endIndex).flatMap((header) => {
      const day = days[header.section];
      if (!day) return [];
      const ids = items.slice(day.start, day.end).map((asset) => asset.id);
      const first = ids[0];
      if (!first) return [];
      return [
        {
          header,
          day,
          first,
          busy: picking !== null && ids.includes(picking),
          title: dayTitle(day.key, today),
          subtitle: daySubtitle(day),
          selected: ids.every((id) => selection.has(id))
        }
      ];
    });
  });

  function indexOf(id: string): number {
    return items.findIndex((asset) => asset.id === id);
  }

  async function selectDay(id: string, extend: boolean): Promise<void> {
    if (locked || picking) return;
    const from = extend && anchor !== null && indexOf(anchor) >= 0 ? anchor : id;
    picking = id;
    try {
      await pickDays(
        {
          pager: browsing,
          groups: () => days,
          indexOf,
          ids: (start, end) => items.slice(start, end).map((asset) => asset.id),
          isSelected: selection.has,
          apply: selection.setMany
        },
        from,
        id
      );
      anchor = id;
    } finally {
      picking = null;
    }
  }
</script>

{#each shown as entry (`${entry.day.key}:${entry.header.start}`)}
  <div
    class="absolute"
    style:top="{pad + entry.header.top}px"
    style:left="{pad}px"
    style:right="{pad}px"
    style:height="{entry.header.height}px"
  >
    <DayHeader
      title={entry.title}
      subtitle={entry.subtitle}
      selected={entry.selected}
      selectionActive={selection.selecting || locked}
      busy={entry.busy}
      onSelect={(extend) => void selectDay(entry.first, extend)}
    />
  </div>
{/each}
