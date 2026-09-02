<script lang="ts">
  import Logo from './Logo.svelte';

  let {
    size = 'medium',
    bright = false,
    class: className = ''
  }: {
    size?: 'small' | 'medium' | 'large';
    bright?: boolean;
    class?: string;
  } = $props();

  const markSize = $derived(size === 'small' ? 26 : size === 'large' ? 40 : 32);
</script>

<span
  class="inline-flex items-center gap-2 leading-none font-bold tracking-tight {className}"
  class:text-lg={size === 'small'}
  class:text-xl={size === 'medium'}
  class:text-2xl={size === 'large'}
  role="img"
  aria-label="immich-edit"
>
  <Logo size={markSize} decorative />
  <span aria-hidden="true" class:text-white={bright} class:text-dark={!bright}
    >immich<span class="text-logo-primary">-edit</span></span
  >
</span>
