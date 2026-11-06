<script lang="ts">
  import ContextMenu from '#lib/components/ContextMenu.svelte';
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import LibraryRow from '#lib/components/library/LibraryRow.svelte';
  import SaveViewForm from '#lib/components/savedViews/SaveViewForm.svelte';
  import type { SavedView } from '#lib/api/savedViews.ts';
  import { isCurrentView, openSavedView } from '#lib/browse/savedViewActions.ts';
  import { parseSavedViewUrl } from '#lib/browse/savedViewUrl.ts';
  import { plainClick } from '#lib/input/modifiers.svelte.ts';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import { mdiBookmark, mdiBookmarkOutline } from '@mdi/js';
  import { tick } from 'svelte';
  import { createAttachmentKey, type Attachment } from 'svelte/attachments';

  let {
    view,
    query = '',
    variant,
    last = false,
    onLink,
    onDelete
  }: {
    view: SavedView;
    query?: string;
    variant: 'sidebar' | 'list';
    last?: boolean;
    onLink: (node: HTMLElement) => () => void;
    onDelete: () => void;
  } = $props();

  let renaming = $state(false);
  let link: HTMLElement | null = null;

  const target = $derived(parseSavedViewUrl(view.url));
  const active = $derived(isCurrentView(view.url));

  const linkKey = createAttachmentKey();
  const capture: Attachment<HTMLElement> = (node) => {
    link = node;
    const release = onLink(node);
    return () => {
      if (link === node) link = null;
      release();
    };
  };

  function onclick(event: MouseEvent): void {
    if (!plainClick(event)) return;
    event.preventDefault();
    void openSavedView(view.url);
  }

  function linkProps(props: Record<string, unknown>): Record<string, unknown> {
    return mergeProps(props, {
      onclick,
      href: target?.path,
      'aria-current': active ? 'page' : undefined,
      [linkKey]: capture
    });
  }

  async function endRename(): Promise<void> {
    renaming = false;
    await tick();
    link?.focus();
  }

  async function rename(name: string): Promise<void> {
    if (await savedViews.rename(view, name)) await endRename();
  }
</script>

{#if renaming}
  <SaveViewForm
    initial={view.name}
    label="Rename {view.name}"
    onSubmit={rename}
    onCancel={() => void endRename()}
    class={variant === 'sidebar' ? 'py-1.5 ps-4 pe-1 wide:px-1' : 'px-4 py-2 sm:px-5'}
  />
{:else}
  <ContextMenu>
    {#snippet trigger(props)}
      {#if variant === 'sidebar'}
        <TooltipLink
          {...linkProps(props)}
          tooltip={view.name}
          class="no-callout flex w-full items-center gap-4 rounded-e-full py-3 ps-5 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary wide:h-11 wide:rounded-[10px] wide:py-0 wide:ps-3 {active
            ? 'bg-primary/10 text-primary'
            : 'hover:bg-subtle hover:text-primary'}"
        >
          <Icon
            icon={active ? mdiBookmark : mdiBookmarkOutline}
            size="1.375em"
            class="shrink-0"
            aria-hidden
          />
          <span data-truncate class="min-w-0 flex-1 truncate text-sm font-medium wide:text-[15px]">
            <HighlightedText text={view.name} {query} />
          </span>
        </TooltipLink>
      {:else}
        <LibraryRow
          {...linkProps(props)}
          icon={mdiBookmarkOutline}
          label={view.name}
          {query}
          {last}
        />
      {/if}
    {/snippet}
    <ContextMenuItem onSelect={() => (renaming = true)}>Rename</ContextMenuItem>
    <ContextMenuItem onSelect={onDelete}>Delete</ContextMenuItem>
  </ContextMenu>
{/if}
