<script lang="ts">
  import { NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import { copyLocationOf, pasteLocationOnto } from '#lib/browse/locationCopy.ts';
  import { locationOf } from '#lib/browse/locationOps.ts';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { locationClipboard } from '#lib/stores/locationClipboard.svelte.ts';
  import { locationPicker } from '#lib/stores/locationPicker.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiContentCopy, mdiContentPaste, mdiMapMarkerOutline } from '@mdi/js';

  let { asset }: { asset: AssetSummary } = $props();

  const owned = $derived(ownsAsset(asset));
  const here = $derived(locationOf(asset.exifInfo));
</script>

<div class="mt-2 flex items-center gap-1">
  <DisabledReason locked={!owned} reason={NOT_OWNER} class="flex">
    <Button
      size="small"
      variant="outline"
      color="secondary"
      leadingIcon={mdiMapMarkerOutline}
      class="touch:h-11"
      disabled={!owned}
      onclick={() => locationPicker.open([asset])}
    >
      {here ? 'Edit location' : 'Set location'}
    </Button>
  </DisabledReason>
  {#if here}
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      icon={mdiContentCopy}
      class="touch:size-11"
      title={hint('Copy location', 'copyLocation')}
      aria-label="Copy location"
      onclick={() => copyLocationOf(asset)}
    />
  {/if}
  {#if locationClipboard.copied}
    <DisabledReason locked={!owned} reason={NOT_OWNER} class="flex">
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiContentPaste}
        class="touch:size-11"
        title={hint('Paste location', 'pasteLocation')}
        aria-label="Paste location"
        disabled={!owned}
        onclick={() => void pasteLocationOnto([asset])}
      />
    </DisabledReason>
  {/if}
</div>
