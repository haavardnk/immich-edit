<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import Dialog from '#lib/components/Dialog.svelte';
  import { locationName } from '#lib/browse/locationCopy.ts';
  import {
    METADATA_FIELDS,
    metadataClipboard,
    type MetadataField
  } from '#lib/stores/metadataClipboard.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  const LABELS: Record<MetadataField, string> = {
    description: 'Description',
    location: 'Location',
    tags: 'Tags',
    rating: 'Rating'
  };

  const clipboard = metadataClipboard;

  function valueOf(field: MetadataField): string | null {
    const copied = clipboard.copied;
    if (!copied) return null;
    if (field === 'description') return copied.description || null;
    if (field === 'location') return copied.location ? locationName(copied.location) : null;
    if (field === 'tags') return copied.tags.map((tag) => tag.name).join(', ') || null;
    return copied.rating ? '★'.repeat(copied.rating) : null;
  }
</script>

{#if clipboard.pasting && clipboard.copied}
  <Dialog
    title="Paste metadata"
    subtitle={countOf(clipboard.pasting.length, 'photo')}
    onClose={clipboard.close}
    bodyClass="flex flex-col gap-3"
  >
    <p class="text-sm text-light-950">From {clipboard.copied.name}</p>
    <div class="flex flex-col" role="group" aria-label="Fields to paste">
      {#each METADATA_FIELDS as field (field)}
        {@const value = valueOf(field)}
        <CheckboxRow
          checked={clipboard.fields.includes(field)}
          label={LABELS[field]}
          disabled={value === null}
          onChange={(on) => clipboard.toggle(field, on)}
        >
          {#if value}
            <span class="ms-auto max-w-56 truncate text-xs text-dark/65 touch:text-sm">
              {value}
            </span>
          {/if}
        </CheckboxRow>
      {/each}
    </div>
    {#snippet footer()}
      <div class="flex w-full items-center justify-end gap-2">
        <Button
          variant="ghost"
          color="secondary"
          class="touch:hidden"
          disabled={clipboard.busy}
          onclick={clipboard.close}
        >
          Cancel
        </Button>
        <Button
          class="touch:h-12 touch:flex-1 touch:rounded-full"
          loading={clipboard.busy}
          disabled={!clipboard.fields.some((field) => valueOf(field) !== null)}
          onclick={clipboard.paste}
        >
          Paste
        </Button>
      </div>
    {/snippet}
  </Dialog>
{/if}
