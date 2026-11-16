<script lang="ts">
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';

  let { class: className = '', dot = false }: { class?: string; dot?: boolean } = $props();

  const host = $derived(immichCapabilities.externalHost);
  const version = $derived(immichCapabilities.version);
</script>

{#if host}
  <p class={className}>
    {#if dot}
      <span
        aria-hidden="true"
        class="me-1.5 inline-block size-1.5 rounded-full bg-success align-middle"
      ></span>
    {/if}{version ? `${host} · Immich v${version}` : host}
  </p>
{/if}
