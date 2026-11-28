<script lang="ts">
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiArrowLeft } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let { title, action, children }: { title: string; action?: Snippet; children: Snippet } =
    $props();
</script>

<section
  aria-label={title}
  class="flex flex-col"
  {@attach (node) => node.scrollIntoView({ block: 'start' })}
>
  <div class="flex h-13 items-center gap-1 border-b border-hairline pr-2 pl-1">
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiArrowLeft}
      aria-label="Back to filters"
      onclick={() => (browseView.filterPage = null)}
    />
    <h3 class="text-base font-semibold text-light-950">{title}</h3>
    {@render action?.()}
  </div>
  <div class="flex flex-col px-4 pt-2 pb-4">
    {@render children()}
  </div>
</section>
