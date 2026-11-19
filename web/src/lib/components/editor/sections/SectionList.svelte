<script lang="ts">
  import { moveWithin } from '#lib/editor/sectionOrder.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { Icon, IconButton, Switch } from '@immich/ui';
  import { mdiDragVertical, mdiEyeOffOutline, mdiEyeOutline, mdiLockOutline } from '@mdi/js';
  import type { LockedSection, SectionGroup, SectionListVariant } from './sectionList';

  let {
    groups,
    locked = [],
    variant,
    onreorder,
    ontoggle
  }: {
    groups: readonly SectionGroup[];
    locked?: readonly LockedSection[];
    variant: SectionListVariant;
    onreorder: (ids: string[]) => void;
    ontoggle: (id: string) => void;
  } = $props();

  type Dragging = { group: string; id: string; from: number; to: number; dy: number; y: number };

  let dragging = $state<Dragging | null>(null);
  let rowHeight = 0;

  const sheet = $derived(variant === 'sheet');
  const rowClass = $derived(sheet ? 'h-11 px-2 text-[15px]' : 'h-8 px-1.5 text-[13px]');

  function offsetOf(group: SectionGroup, index: number, id: string): number {
    const current = dragging;
    if (!current || current.group !== group.id) return 0;
    if (current.id === id) return current.dy;
    if (current.from < current.to && index > current.from && index <= current.to) return -rowHeight;
    if (current.to < current.from && index >= current.to && index < current.from) return rowHeight;
    return 0;
  }

  const drag = pointerDrag<Dragging, { group: SectionGroup; id: string }>({
    start: (e, { group, id }) => {
      if (group.rows.length < 2 || !(e.currentTarget instanceof Element)) return null;
      rowHeight =
        e.currentTarget.closest('[data-section-row]')?.getBoundingClientRect().height ?? 0;
      const from = group.rows.findIndex((row) => row.id === id);
      e.preventDefault();
      dragging = { group: group.id, id, from, to: from, dy: 0, y: e.clientY };
      return dragging;
    },
    move: (e, state) => {
      if (!dragging || rowHeight <= 0) return;
      const count = groups.find((group) => group.id === state.group)?.rows.length ?? 1;
      const dy = e.clientY - state.y;
      const to = Math.max(0, Math.min(count - 1, state.from + Math.round(dy / rowHeight)));
      dragging = { ...dragging, dy, to };
    },
    end: () => {
      const done = dragging;
      dragging = null;
      if (!done || done.to === done.from) return;
      const order = groups.find((group) => group.id === done.group)?.rows.map((row) => row.id);
      if (order) onreorder(moveWithin(order, done.id, done.to));
    },
    cancel: () => {
      dragging = null;
    }
  });

  function step(e: KeyboardEvent, group: SectionGroup, id: string): void {
    const delta = e.key === 'ArrowUp' ? -1 : e.key === 'ArrowDown' ? 1 : 0;
    if (delta === 0) return;
    e.preventDefault();
    const order = group.rows.map((row) => row.id);
    const index = order.indexOf(id) + delta;
    if (index < 0 || index >= order.length) return;
    onreorder(moveWithin(order, id, index));
  }
</script>

<div class="flex flex-col">
  {#each locked as section (section.label)}
    <div class="flex items-center gap-2.5 text-dark/65 {rowClass}">
      <Icon icon={mdiLockOutline} size="15px" class="ms-1.5 shrink-0" aria-hidden="true" />
      <span class="min-w-0 flex-1 truncate">{section.label}</span>
      <span class="shrink-0 text-[11px] text-dark/55">{section.note}</span>
    </div>
  {/each}
  {#each groups as group (group.id)}
    {#if group.label}
      <h3 class="px-2 pt-3 pb-1 text-[11px] font-medium tracking-wider text-dark/65 uppercase">
        {group.label}
      </h3>
    {/if}
    <ul data-section-group aria-label={group.label ?? 'Sections'} class="flex flex-col">
      {#each group.rows as row, index (row.id)}
        <li
          data-section-row
          data-hidden={row.hidden || undefined}
          style:translate={dragging?.group === group.id
            ? `0 ${offsetOf(group, index, row.id)}px`
            : undefined}
          class="flex items-center gap-1.5 rounded-lg {rowClass} {dragging?.id === row.id
            ? 'relative z-10 bg-light-200 shadow-popover'
            : dragging?.group === group.id
              ? 'transition-[translate] duration-150'
              : ''}"
        >
          <IconButton
            size="tiny"
            variant="ghost"
            color="secondary"
            icon={mdiDragVertical}
            title="Drag or use the arrow keys to reorder"
            aria-label="Reorder {row.label}"
            disabled={group.rows.length < 2}
            class="shrink-0 cursor-grab touch-none {sheet ? 'size-9' : ''}"
            onpointerdown={(e: PointerEvent) => drag.begin(e, { group, id: row.id })}
            onpointermove={drag.follow.onpointermove}
            onpointerup={drag.follow.onpointerup}
            onpointercancel={drag.follow.onpointercancel}
            onlostpointercapture={drag.follow.onlostpointercapture}
            onkeydown={(e: KeyboardEvent) => step(e, group, row.id)}
          />
          <span class="min-w-0 flex-1 truncate {row.hidden ? 'text-dark/45' : 'text-dark'}">
            {row.label}
          </span>
          {#if sheet}
            <Switch
              checked={!row.hidden}
              onCheckedChange={() => ontoggle(row.id)}
              aria-label="Show {row.label}"
            />
          {:else}
            {#if row.hidden}
              <span class="shrink-0 text-[11px] text-dark/55">hidden</span>
            {/if}
            <IconButton
              size="tiny"
              variant="ghost"
              color="secondary"
              icon={row.hidden ? mdiEyeOffOutline : mdiEyeOutline}
              title={row.hidden ? `Show ${row.label}` : `Hide ${row.label}`}
              aria-label={row.hidden ? `Show ${row.label}` : `Hide ${row.label}`}
              class="shrink-0"
              onclick={() => ontoggle(row.id)}
            />
          {/if}
        </li>
      {/each}
    </ul>
  {/each}
</div>
