<script lang="ts">
  import { pointerKind } from '#lib/device/pointerKind.svelte.ts';
  import { Tooltip } from 'bits-ui';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<Tooltip.Provider disableCloseOnTriggerClick ignoreNonKeyboardFocus disabled={pointerKind.touch}>
  {@render children()}
</Tooltip.Provider>
