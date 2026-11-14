<script lang="ts">
  import { copyPath } from '#lib/browse/info/copyPath.ts';
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button } from '@immich/ui';
  import { mdiContentCopy } from '@mdi/js';

  let { asset }: { asset: AssetSummary } = $props();

  const path = $derived(asset.originalPath ?? asset.originalFileName);
</script>

<InfoSection title="File">
  <p class="font-mono text-[11px] break-all text-light-950/80 touch:text-[13px]">{path}</p>
  {#if asset.originalPath}
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      leadingIcon={mdiContentCopy}
      class="-ms-2 h-7 w-fit px-2 text-xs touch:h-11 touch:text-sm"
      onclick={() => asset.originalPath && void copyPath(asset.originalPath)}
    >
      Copy path
    </Button>
  {/if}
</InfoSection>
