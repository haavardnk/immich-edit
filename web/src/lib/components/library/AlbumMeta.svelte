<script lang="ts">
  import { albumDateRange, roleLine } from '#lib/browse/albumSharing.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import type { AlbumSummary } from '#lib/types/album.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiAccountMultiple, mdiHistory } from '@mdi/js';
  import type { Attachment } from 'svelte/attachments';

  let { album }: { album: AlbumSummary } = $props();

  const role = $derived(roleLine(album, session.user?.id ?? null));
  const dates = $derived(albumDateRange(album));
  const description = $derived(album.description.trim());

  let expanded = $state(false);
  let overflows = $state(false);

  function measure(text: string): Attachment<HTMLElement> {
    return (node) => {
      expanded = false;
      const check = (): void => {
        if (!node.classList.contains('line-clamp-2')) return;
        overflows = text.length > 0 && node.scrollHeight > node.clientHeight + 1;
      };
      const observer = new ResizeObserver(check);
      observer.observe(node);
      check();
      return () => observer.disconnect();
    };
  }
</script>

{#if role || dates || description}
  <div class="flex flex-col gap-1 px-3 pt-1 pb-2 text-xs text-dark/65 sm:px-5 touch:text-label">
    {#if role || dates}
      <p class="flex flex-wrap items-center gap-x-4 gap-y-1">
        {#if dates}
          <span class="inline-flex items-center gap-1.5">
            <Icon icon={mdiHistory} size="14px" aria-hidden="true" />
            {dates}
          </span>
        {/if}
        {#if role}
          <span class="inline-flex items-center gap-1.5">
            <Icon icon={mdiAccountMultiple} size="14px" aria-hidden="true" />
            {role}
          </span>
        {/if}
      </p>
    {/if}
    {#if description}
      <div class="flex items-end gap-2">
        <p
          id="album-description"
          class="min-w-0 flex-1 whitespace-pre-line text-dark/85 {expanded ? '' : 'line-clamp-2'}"
          {@attach measure(description)}
        >
          {description}
        </p>
        {#if overflows}
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            class="shrink-0"
            aria-expanded={expanded}
            aria-controls="album-description"
            onclick={() => (expanded = !expanded)}
          >
            {expanded ? 'Less' : 'More'}
          </Button>
        {/if}
      </div>
    {/if}
  </div>
{/if}
