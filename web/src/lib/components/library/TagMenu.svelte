<script lang="ts">
  import { TAG_ACTIONS, tagMenu, type TagMenuPlace } from '#lib/browse/tagActions.ts';
  import type { TagNode } from '#lib/browse/tagTree.ts';
  import ActionMenu from '#lib/components/ActionMenu.svelte';
  import { tagActions } from '#lib/stores/tagActions.svelte.ts';

  let {
    node,
    place,
    variant,
    onrun
  }: {
    node: TagNode;
    place: TagMenuPlace;
    variant: 'context' | 'popover';
    onrun?: () => void;
  } = $props();
</script>

<ActionMenu
  groups={tagMenu(place)}
  actions={TAG_ACTIONS}
  {variant}
  onrun={(action) => {
    onrun?.();
    void tagActions.run(action, node);
  }}
/>
