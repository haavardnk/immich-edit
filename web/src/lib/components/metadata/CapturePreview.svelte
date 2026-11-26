<script lang="ts">
  import { thumbUrl } from '#lib/api/assets.ts';
  import Chip from '#lib/components/chips/Chip.svelte';
  import { sameDay, shownText, zoneName } from '#lib/metadata/captureLabels.ts';
  import type { PreviewRow } from '#lib/metadata/captureTimes.ts';
  import { captureEditor, type PreviewFilter } from '#lib/stores/captureEditor.svelte.ts';
  import { Button } from '@immich/ui';

  const SHOWN = 300;

  const editor = captureEditor;
  const several = $derived(editor.cameras.length > 1);
  const picking = $derived(editor.mode === 'photo');
  const shown = $derived(editor.visible.slice(0, SHOWN));
  const chips: { id: PreviewFilter; label: string }[] = $derived([
    { id: 'all', label: `All ${editor.rows.length}` },
    ...(editor.changed > 0 ? [{ id: 'changed' as const, label: `Changed ${editor.changed}` }] : []),
    ...(editor.reordered > 0
      ? [{ id: 'reordered' as const, label: `Reordered ${editor.reordered}` }]
      : [])
  ]);
</script>

{#snippet rowBody(row: PreviewRow)}
  {@const withDay = !sameDay(row.before, row.after) || editor.mode === 'once'}
  <img
    src={thumbUrl(row.photo.id, 'thumbnail')}
    alt=""
    loading="lazy"
    class="size-10 shrink-0 rounded object-cover touch:size-11"
  />
  <span class="min-w-0 flex-1">
    <span class="block truncate text-xs text-light-950 touch:text-sm">{row.photo.name}</span>
    {#if several}
      <span class="block truncate text-caption text-dark/65">
        {editor.labelOf(row.photo)}
      </span>
    {/if}
  </span>
  <span class="flex shrink-0 flex-col items-end text-xs tabular-nums touch:text-sm">
    {#if row.changed}
      <span class="font-semibold text-light-950">
        {shownText(row.after, withDay)}
        {#if row.zone && row.zone !== row.photo.zone}
          <span class="font-normal text-dark/65">{zoneName(row.zone)}</span>
        {/if}
      </span>
      <span class="text-caption text-dark/65">was {shownText(row.before, withDay)}</span>
    {:else}
      <span class="text-dark/80">{shownText(row.before, withDay)}</span>
    {/if}
  </span>
  <span class="w-24 shrink-0 text-end touch:hidden">
    {#if row.reordered}
      <span class="rounded-full bg-warning-500/10 px-2 py-0.5 text-caption text-warning-800">
        Reordered
      </span>
    {/if}
  </span>
{/snippet}

<section class="flex min-h-0 flex-1 flex-col" aria-label="Preview">
  <div class="flex shrink-0 flex-wrap items-center gap-1.5 px-4 pt-3 pb-2 touch:px-0">
    <h3 class="section-label me-auto text-dark/65">Preview</h3>
    {#each chips as chip (chip.id)}
      <Chip
        pressed={editor.shownFilter === chip.id}
        onPressedChange={(on) => (editor.filter = on ? chip.id : 'all')}
      >
        {chip.label}
      </Chip>
    {/each}
  </div>
  <ul class="flex min-h-0 flex-1 flex-col overflow-y-auto pb-2 touch:@max-3xl:overflow-visible">
    {#each shown as row (row.photo.id)}
      {@const current = picking && editor.picked?.id === row.photo.id}
      <li class={row.changed || current ? '' : 'opacity-50'} data-capture-row={row.photo.id}>
        {#if picking}
          <Button
            size="tiny"
            variant="ghost"
            color={current ? 'primary' : 'secondary'}
            class="h-auto w-full justify-start gap-3 rounded-none px-4 py-1.5 text-start font-normal -outline-offset-2 touch:px-0 {current
              ? 'bg-primary/10'
              : ''}"
            aria-label="Pick {row.photo.name}"
            aria-pressed={current}
            onclick={() => editor.pickPhoto(row.photo.id)}
          >
            {@render rowBody(row)}
          </Button>
        {:else}
          <div class="flex items-center gap-3 px-4 py-1.5 touch:px-0">
            {@render rowBody(row)}
          </div>
        {/if}
      </li>
    {/each}
    {#if editor.visible.length > SHOWN}
      <li class="px-4 py-2 text-caption text-dark/65 touch:px-0">
        and {editor.visible.length - SHOWN} more
      </li>
    {/if}
  </ul>
</section>
