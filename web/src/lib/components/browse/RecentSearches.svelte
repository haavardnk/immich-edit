<script lang="ts">
  import SectionHeading from '#lib/components/SectionHeading.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { recentSearches } from '#lib/stores/recentSearches.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import { mdiClose, mdiHistory } from '@mdi/js';

  let { onRun }: { onRun: (query: string) => void } = $props();
</script>

{#snippet clear()}
  <Button
    size="tiny"
    variant="ghost"
    color="primary"
    class="-me-2 touch:min-h-11 touch:text-[12.5px]"
    onclick={recentSearches.clear}
  >
    Clear
  </Button>
{/snippet}

{#if recentSearches.items.length > 0}
  <section aria-labelledby="recent-searches" class="w-full max-w-md text-left">
    <SectionHeading id="recent-searches" title="Recent" action={clear} />
    <ul class="flex flex-col px-2 sm:px-3">
      {#each recentSearches.items as query (query)}
        <li class="flex items-center gap-1">
          <TruncationTooltip text={query}>
            {#snippet child({ props })}
              <Button
                {...mergeProps(props, { onclick: () => onRun(query) })}
                size="small"
                variant="ghost"
                color="secondary"
                class="min-w-0 flex-1 justify-start gap-3.5 font-normal touch:min-h-11 touch:text-[15px] [&_svg]:text-dark/65"
                leadingIcon={mdiHistory}
                title=""
              >
                <span data-truncate class="truncate">{query}</span>
              </Button>
            {/snippet}
          </TruncationTooltip>
          <IconButton
            size="small"
            variant="ghost"
            color="secondary"
            shape="round"
            class="text-dark/65 touch:size-11"
            icon={mdiClose}
            aria-label={`Remove ${query} from recent searches`}
            onclick={() => recentSearches.remove(query)}
          />
        </li>
      {/each}
    </ul>
  </section>
{/if}
