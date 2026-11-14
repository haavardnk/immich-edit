<script lang="ts">
  import { moveWithin } from '#lib/editor/sectionOrder.ts';
  import { Reorder } from '#lib/gestures/reorder.svelte.ts';
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

  const reorder = new Reorder(({ group, id, to }) => {
    const order = groups.find((entry) => entry.id === group)?.rows.map((row) => row.id);
    if (order) onreorder(moveWithin(order, id, to));
  });

  const sheet = $derived(variant === 'sheet');
  const rowClass = $derived(sheet ? 'h-11 px-2 text-[15px]' : 'h-8 px-1.5 text-[13px]');

  function idsOf(group: SectionGroup): string[] {
    return group.rows.map((row) => row.id);
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
      {#each group.rows as row (row.id)}
        <li
          data-section-row
          data-reorder-item
          data-hidden={row.hidden || undefined}
          style:translate={reorder.active(group.id)
            ? `0 ${reorder.offset(group.id, idsOf(group), row.id)}px`
            : undefined}
          class="flex items-center gap-1.5 rounded-lg {rowClass} {reorder.dragging?.id === row.id
            ? 'relative z-10 bg-light-200 shadow-popover'
            : reorder.active(group.id)
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
            {...reorder.handle(group.id, idsOf(group), row.id)}
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
