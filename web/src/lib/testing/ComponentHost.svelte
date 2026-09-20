<script lang="ts" generics="T extends Record<string, unknown>">
  import type { Component } from 'svelte';
  import { TooltipProvider } from '@immich/ui';

  let { component: Child, props }: { component: Component<T>; props: T } = $props();
</script>

<TooltipProvider>
  <Child {...props} />
</TooltipProvider>
