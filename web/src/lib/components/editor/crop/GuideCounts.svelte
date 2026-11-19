<script lang="ts">
  import { countText, GUIDE_KINDS } from '#lib/utils/uprightFeedback.ts';
  import type { UprightGuide } from '#lib/utils/upright.ts';

  let { guides }: { guides: readonly UprightGuide[] } = $props();
</script>

<span class="inline-flex flex-wrap items-center gap-x-2">
  {#each GUIDE_KINDS as kind, i (kind)}
    {#if i > 0}<span aria-hidden="true">·</span>{/if}
    <span class="inline-flex items-center gap-1.5"
      ><span
        class="size-2 rounded-full {kind === 'vertical'
          ? 'bg-guide-vertical'
          : 'bg-guide-horizontal'}"
        aria-hidden="true"
      ></span>{countText(guides, kind)}</span
    >
  {/each}
</span>
