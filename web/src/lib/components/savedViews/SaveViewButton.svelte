<script lang="ts">
  import { Button, IconButton } from '@immich/ui';
  import { mdiBookmarkOutline } from '@mdi/js';
  import type { Attachment } from 'svelte/attachments';

  let {
    open,
    controls,
    iconOnly = false,
    onRef,
    onclick
  }: {
    open: boolean;
    controls: string;
    iconOnly?: boolean;
    onRef: (node: HTMLElement | null) => void;
    onclick: () => void;
  } = $props();

  const capture: Attachment<HTMLElement> = (node) => {
    onRef(node);
    return () => onRef(null);
  };
</script>

{#if iconOnly}
  <IconButton
    {@attach capture}
    size="small"
    variant="ghost"
    color={open ? 'primary' : 'secondary'}
    icon={mdiBookmarkOutline}
    title="Save view"
    aria-label="Save view"
    aria-expanded={open}
    aria-controls={open ? controls : undefined}
    class="min-h-11 min-w-11 flex-none {open ? 'bg-primary/10' : ''}"
    {onclick}
  />
{:else}
  <Button
    {@attach capture}
    size="tiny"
    variant="ghost"
    color={open ? 'primary' : 'secondary'}
    class="h-8 shrink-0 whitespace-nowrap {open
      ? 'bg-primary/10'
      : 'bg-transparent hover:bg-light-950/6'}"
    leadingIcon={mdiBookmarkOutline}
    aria-expanded={open}
    aria-controls={open ? controls : undefined}
    {onclick}
  >
    Save view
  </Button>
{/if}
