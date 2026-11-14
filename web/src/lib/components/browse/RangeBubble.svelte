<script lang="ts">
  import type { AssetGridBox } from '#lib/browse/assetGridLayout.ts';

  let { box, pad, added }: { box: AssetGridBox; pad: number; added: number } = $props();
</script>

<div
  role="status"
  class="pointer-events-none absolute z-30 -translate-x-1/2 -translate-y-full rounded-full bg-light-50/95 px-3 py-1 text-[13px] font-semibold whitespace-nowrap text-light-950 shadow-popover ring-1 ring-hairline backdrop-blur-md"
  style:left="{pad + box.left + box.width / 2}px"
  style:top="{pad + box.top + box.height - 8}px"
>
  +{added} in range
</div>
