<script lang="ts">
  import { recentSearches } from '#lib/stores/recentSearches.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiHistory } from '@mdi/js';

  let { onRun }: { onRun: (query: string) => void } = $props();
</script>

{#if recentSearches.items.length > 0}
  <section aria-labelledby="recent-searches" class="w-full max-w-md text-left">
    <div class="flex items-center justify-between ps-3">
      <h2 id="recent-searches" class="text-xs font-semibold text-dark/65">Recent</h2>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="touch:min-h-11"
        onclick={recentSearches.clear}
      >
        Clear
      </Button>
    </div>
    <ul class="flex flex-col">
      {#each recentSearches.items as query (query)}
        <li class="flex items-center gap-1">
          <Button
            size="small"
            variant="ghost"
            color="secondary"
            class="min-w-0 flex-1 justify-start gap-3 font-normal touch:min-h-11"
            leadingIcon={mdiHistory}
            onclick={() => onRun(query)}
          >
            <span class="truncate">{query}</span>
          </Button>
          <IconButton
            size="small"
            variant="ghost"
            color="secondary"
            shape="round"
            class="touch:size-11"
            icon={mdiClose}
            aria-label={`Remove ${query} from recent searches`}
            onclick={() => recentSearches.remove(query)}
          />
        </li>
      {/each}
    </ul>
  </section>
{/if}
