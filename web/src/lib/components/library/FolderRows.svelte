<script lang="ts">
  import { folderHref, parentFolder } from '#lib/browse/folders.ts';
  import HighlightedText from '#lib/components/HighlightedText.svelte';
  import TooltipLink from '#lib/components/TooltipLink.svelte';
  import type { FolderNode } from '#lib/stores/library.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight, mdiFolderOutline } from '@mdi/js';

  let { folders, query = '' }: { folders: readonly FolderNode[]; query?: string } = $props();
</script>

{#if folders.length > 0}
  <ul aria-label="Subfolders" class="flex flex-col">
    {#each folders as folder, index (folder.path)}
      <li>
        <TooltipLink
          tooltip={query ? folder.path : folder.name}
          href={folderHref(folder.path)}
          class="flex min-h-10 items-center gap-3 ps-3 pe-2.5 outline-none hover:bg-subtle focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary sm:ps-5 touch:min-h-14 touch:gap-3.5 touch:ps-4"
        >
          <Icon
            icon={mdiFolderOutline}
            size="20px"
            class="shrink-0 text-primary touch:size-6"
            aria-hidden="true"
          />
          <span
            class="flex min-h-10 min-w-0 flex-1 items-center gap-2 self-stretch touch:min-h-14 {index <
            folders.length - 1
              ? 'border-b border-hairline'
              : ''}"
          >
            <span class="min-w-0 flex-1">
              <span data-truncate class="block truncate text-sm text-light-950 touch:text-[15.5px]">
                <HighlightedText text={folder.name} {query} />
              </span>
              {#if query}
                {#if parentFolder(folder.path)}
                  <span class="block truncate text-xs text-dark/65 touch:text-[12.5px]">
                    {parentFolder(folder.path)}
                  </span>
                {/if}
              {:else if folder.children.length > 0}
                <span class="block text-xs text-dark/65 touch:text-[12.5px]">
                  {countOf(folder.children.length, 'folder')}
                </span>
              {/if}
            </span>
            <Icon
              icon={mdiChevronRight}
              size="18px"
              class="shrink-0 text-dark/65"
              aria-hidden="true"
            />
          </span>
        </TooltipLink>
      </li>
    {/each}
  </ul>
{/if}
