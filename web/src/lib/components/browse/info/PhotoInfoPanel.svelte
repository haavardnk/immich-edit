<script lang="ts">
  import PhotoInfoBody from '#lib/components/browse/info/PhotoInfoBody.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let { asset, onclose }: { asset: AssetSummary; onclose: () => void } = $props();
</script>

<aside
  aria-label="Info"
  class="flex w-90 shrink-0 flex-col border-l border-hairline bg-editor-chrome text-light-950/90"
>
  <div class="flex h-11 shrink-0 items-center gap-2 border-b border-hairline pr-1.5 pl-4">
    <h2 class="min-w-0 flex-1 truncate text-label font-semibold">Info</h2>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiClose}
      title={hint('Close', 'toggleInfo')}
      aria-label="Close info"
      onclick={onclose}
    />
  </div>
  <div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
    <PhotoInfoBody {asset} />
  </div>
</aside>
