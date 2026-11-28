<script lang="ts">
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let { onclick, children }: { onclick: () => void; children: Snippet } = $props();
</script>

<Button
  size="medium"
  color="primary"
  shape="round"
  class="h-12 w-full text-base font-semibold"
  {onclick}
>
  {@render children()}
</Button>
