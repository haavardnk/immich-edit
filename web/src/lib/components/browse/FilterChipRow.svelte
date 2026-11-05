<script lang="ts">
  import { browseControls, type FilterKey } from '#lib/stores/browseControls.svelte.ts';
  import { LABEL_TEXT } from '#lib/stores/labels.ts';
  import { activeFilterChips, withoutFilter } from '#lib/browse/filterChips.ts';
  import { idName } from '#lib/browse/idOptions.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiCircle, mdiClose, mdiTrashCanOutline } from '@mdi/js';

  let { onTrashRejected }: { onTrashRejected?: () => Promise<void> } = $props();

  let trashBusy = $state(false);

  const chips = $derived(activeFilterChips(browseControls.filters, idName));

  async function trashRejected(): Promise<void> {
    if (!onTrashRejected || trashBusy) return;
    trashBusy = true;
    try {
      await onTrashRejected();
    } finally {
      trashBusy = false;
    }
  }

  function removeFilter(key: FilterKey): void {
    browseControls.applyFilters(withoutFilter(browseControls.filters, key));
  }
</script>

{#if chips.length > 0}
  <div
    role="list"
    aria-label="Active filters"
    class="flex shrink-0 flex-wrap items-center gap-1.5 border-b border-hairline bg-light px-3 py-1.5 sm:px-5"
  >
    {#each chips as chip (chip.key)}
      <span
        role="listitem"
        class="inline-flex h-6 shrink-0 items-center gap-1 rounded-full bg-primary-200 pl-2.5 text-[11px] whitespace-nowrap touch:h-8 touch:pl-3 touch:text-xs"
      >
        {#if chip.color}
          <Icon icon={mdiCircle} size="10px" class={LABEL_TEXT[chip.color]} aria-hidden="true" />
        {/if}
        {chip.label}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          class="opacity-60 hover:opacity-100 touch:-my-1.5 touch:size-11"
          icon={mdiClose}
          title={`Remove ${chip.label}`}
          aria-label={`Remove ${chip.label}`}
          onclick={() => removeFilter(chip.key)}
        />
      </span>
    {/each}
    {#if onTrashRejected && browseControls.filters.rejected === 'only'}
      <Button
        size="tiny"
        variant="ghost"
        color="danger"
        class="ms-auto"
        leadingIcon={mdiTrashCanOutline}
        title="Load every rejected photo here and move them to the Immich trash"
        loading={trashBusy}
        disabled={trashBusy}
        onclick={() => void trashRejected()}
      >
        Move rejected to trash
      </Button>
    {/if}
  </div>
{/if}
