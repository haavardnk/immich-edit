<script lang="ts" generics="T extends Record<string, unknown>">
  import type { Component } from 'svelte';
  import AppTooltipProvider from '#lib/components/AppTooltipProvider.svelte';

  let { component: Child, props }: { component: Component<T>; props: T } = $props();
</script>

<AppTooltipProvider>
  <Child {...props} />
</AppTooltipProvider>
