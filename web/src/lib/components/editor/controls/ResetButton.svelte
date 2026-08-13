<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';

  let {
    title,
    label = title,
    onclick
  }: {
    title: string;
    label?: string;
    onclick: (e: MouseEvent) => void;
  } = $props();
</script>

<button
  type="button"
  class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
  {title}
  aria-label={label}
  {onclick}
>
  <Icon path={mdiRestore} size={14} />
</button>
