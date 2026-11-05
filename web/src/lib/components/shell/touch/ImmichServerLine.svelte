<script lang="ts">
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';

  let { class: className = '' }: { class?: string } = $props();

  const host = $derived(immichCapabilities.externalHost);
  const version = $derived(immichCapabilities.version);
</script>

{#if host}
  <p class={className}>{version ? `${host} · Immich v${version}` : host}</p>
{/if}
