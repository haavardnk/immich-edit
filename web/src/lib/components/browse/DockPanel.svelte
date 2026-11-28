<script lang="ts">
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    title,
    closeId,
    closeLabel,
    closeTitle,
    onClose,
    badge,
    actions,
    children
  }: {
    title: string;
    closeId: string;
    closeLabel: string;
    closeTitle: string;
    onClose: () => void;
    badge?: Snippet;
    actions?: Snippet;
    children: Snippet;
  } = $props();
</script>

<div
  class="flex h-12 shrink-0 items-center gap-2 border-b border-hairline pr-1.5 pl-3 touch:h-13 touch:pl-4"
>
  <h2
    class="flex flex-1 items-center gap-1.5 text-label font-semibold whitespace-nowrap text-light-950/90 touch:text-base"
  >
    {title}
    {@render badge?.()}
  </h2>
  {@render actions?.()}
  <IconButton
    id={closeId}
    size="small"
    variant="ghost"
    color="secondary"
    icon={mdiClose}
    title={closeTitle}
    aria-label={closeLabel}
    onclick={onClose}
  />
</div>
<div class="min-h-0 flex-1 overflow-y-auto scrollbar-hidden">
  {@render children()}
</div>
