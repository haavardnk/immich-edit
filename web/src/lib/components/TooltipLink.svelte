<script lang="ts">
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
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
</script>

{#snippet anchor({ props }: { props: Record<string, unknown> })}
  <a {...mergeProps(props, rest)}>{@render children()}</a>
{/snippet}

<TruncationTooltip text={tooltip} force={always} child={anchor} />
