<script lang="ts">
  import type { TagMenuPlace } from '#lib/browse/tagActions.ts';
  import { tagCaption } from '#lib/browse/tagDialogText.ts';
  import { descendantCount, type TagNode } from '#lib/browse/tagTree.ts';
  import ContextMenu from '#lib/components/ContextMenu.svelte';
  import TagMenu from '#lib/components/library/TagMenu.svelte';
  import type { Snippet } from 'svelte';

  let {
    node,
    place,
    trigger
  }: {
    node: TagNode;
    place: TagMenuPlace;
    trigger: Snippet<[Record<string, unknown>]>;
  } = $props();
</script>

<ContextMenu
  title={node.name}
  subtitle={node.tag ? tagCaption(node.tag, descendantCount(node)) : undefined}
  disabled={!node.tag}
  {trigger}
>
  <TagMenu {node} {place} variant="context" />
</ContextMenu>
