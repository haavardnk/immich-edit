<script lang="ts">
  import { NOT_OWNER, ownsAsset } from '#lib/browse/ownership.ts';
  import { locationOf } from '#lib/browse/locationOps.ts';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import { locationPicker } from '#lib/stores/locationPicker.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiContentCopy, mdiContentPaste, mdiMapMarkerOutline } from '@mdi/js';

  let { asset }: { asset: AssetSummary } = $props();

  const owned = $derived(ownsAsset(asset));
  const here = $derived(locationOf(asset.exifInfo));

  function copy(): void {
    locationPicker.copy(asset);
    toasts.push('info', 'Location copied');
  }
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
      title="Copy location"
      aria-label="Copy location"
      onclick={copy}
    />
  {/if}
  {#if locationPicker.copied}
    <DisabledReason locked={!owned} reason={NOT_OWNER} class="flex">
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiContentPaste}
        class="touch:size-11"
        title="Paste location"
        aria-label="Paste location"
        disabled={!owned}
        onclick={() => void locationPicker.paste([asset])}
      />
    </DisabledReason>
  {/if}
</div>
