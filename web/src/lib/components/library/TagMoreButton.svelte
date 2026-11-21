<script lang="ts">
  import type { TagMenuPlace } from '#lib/browse/tagActions.ts';
  import { tagCaption } from '#lib/browse/tagDialogText.ts';
  import { descendantCount, type TagNode } from '#lib/browse/tagTree.ts';
  import MoreActionsButton from '#lib/components/MoreActionsButton.svelte';
  import TagMenu from '#lib/components/library/TagMenu.svelte';

  let {
    node,
    place,
    size = 'small',
    class: className = ''
  }: {
    node: TagNode;
    place: TagMenuPlace;
    size?: 'tiny' | 'small';
    class?: string;
  } = $props();
</script>

<MoreActionsButton
  label="Tag actions"
  title={node.name}
  subtitle={node.tag ? tagCaption(node.tag, descendantCount(node)) : undefined}
  {size}
  class={className}
>
  {#snippet children(close)}
    <TagMenu {node} {place} variant="popover" onrun={close} />
  {/snippet}
</MoreActionsButton>
