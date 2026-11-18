<script lang="ts">
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import { renderer, type RendererChoice } from '#lib/stores/renderer.svelte.ts';
  import { rendererStatus } from '#lib/stores/rendererStatus.ts';

  const OPTIONS: { value: RendererChoice; label: string }[] = [
    { value: 'auto', label: 'Fastest' },
    { value: 'browser', label: 'Browser' },
    { value: 'server', label: 'Server' }
  ];

  const status = $derived(rendererStatus(renderer));
</script>

<section class="space-y-2 py-2">
  <SegmentedChoice
    options={OPTIONS}
    value={renderer.choice}
    label="Preview renderer"
    onChange={(choice) => renderer.setChoice(choice)}
    {status}
  >
    {#if renderer.choice !== 'server' && !isSecureContext}
      <a
        class="text-xs text-primary hover:underline"
        href="https://haavardnk.github.io/immich-edit/rendering/#browser-previews-on-a-local-network"
        target="_blank"
        rel="noopener">Enable browser previews on a local network</a
      >
    {/if}
  </SegmentedChoice>
</section>
