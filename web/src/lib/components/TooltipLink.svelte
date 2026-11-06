<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import type { HTMLAnchorAttributes } from 'svelte/elements';

  let {
    tooltip,
    always = false,
    children,
    ...rest
  }: { tooltip: string; always?: boolean; children: Snippet } & Omit<
    HTMLAnchorAttributes,
    'children'
  > = $props();

  function anchorProps(props: Record<string, unknown>): Record<string, unknown> {
    return { ...mergeProps(props, rest), type: rest.type ?? null };
  }
</script>

{#snippet anchor({ props }: { props: Record<string, unknown> })}
  <a {...anchorProps(props)}>{@render children()}</a>
{/snippet}

{#if always}
  <Tooltip text={tooltip} child={anchor} />
{:else}
  <TruncationTooltip text={tooltip} child={anchor} />
{/if}
