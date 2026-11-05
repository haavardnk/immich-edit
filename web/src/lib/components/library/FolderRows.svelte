<script lang="ts">
  import { folderHref } from '#lib/browse/folders.ts';
  import type { FolderNode } from '#lib/stores/library.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Icon } from '@immich/ui';
  import { mdiChevronRight, mdiFolderOutline } from '@mdi/js';

  let { folders }: { folders: readonly FolderNode[] } = $props();
</script>

{#if folders.length > 0}
  <ul aria-label="Subfolders" class="flex flex-col border-b border-hairline">
    {#each folders as folder (folder.path)}
      <li>
        <a
          href={folderHref(folder.path)}
          class="flex min-h-10 items-center gap-3 px-3 outline-none hover:bg-subtle focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary sm:px-5 touch:min-h-13"
        >
          <Icon
            icon={mdiFolderOutline}
            size="20px"
            class="shrink-0 text-dark/65"
            aria-hidden="true"
          />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-sm text-light-950">{folder.name}</span>
            {#if folder.children.length > 0}
              <span class="block text-xs text-dark/65">
                {countOf(folder.children.length, 'folder')}
              </span>
            {/if}
          </span>
          <Icon
            icon={mdiChevronRight}
            size="18px"
            class="shrink-0 text-dark/40"
            aria-hidden="true"
          />
        </a>
      </li>
    {/each}
  </ul>
{/if}
