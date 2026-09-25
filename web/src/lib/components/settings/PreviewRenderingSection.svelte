<script lang="ts">
  import { RadioGroup } from 'bits-ui';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { renderer, type RendererChoice } from '$lib/stores/renderer.svelte';

  const OPTIONS: { value: RendererChoice; label: string }[] = [
    { value: 'auto', label: 'Browser when available' },
    { value: 'server', label: 'Server' }
  ];

  const status = $derived.by(() => {
    if (renderer.choice === 'server') return 'Previews render on the server.';
    if (renderer.state === 'browser') {
      return `Previews render in this browser on ${renderer.adapter || 'the GPU'}.`;
    }
    if (renderer.state === 'server') return `Previews render on the server: ${renderer.reason}`;
    return 'The browser renderer starts with the editor.';
  });
</script>

<section class="space-y-2 py-2">
  <RadioGroup.Root
    value={renderer.choice}
    onValueChange={(value) => {
      const picked = OPTIONS.find((option) => option.value === value);
      if (picked) renderer.setChoice(picked.value);
    }}
    orientation="horizontal"
    aria-label="Preview renderer"
    class="{segmentedControlClass} grid w-fit grid-cols-2"
  >
    {#each OPTIONS as option (option.value)}
      <RadioGroup.Item value={option.value} class="{segmentedRadioItemClass} px-3">
        {option.label}
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>
  <p class="text-xs text-dark/65">{status}</p>
  {#if renderer.choice === 'auto' && !isSecureContext}
    <a
      class="text-xs text-primary hover:underline"
      href="https://haavardnk.github.io/immich-edit/rendering/#browser-previews-on-a-local-network"
      target="_blank"
      rel="noopener">Enable browser previews on a local network</a
    >
  {/if}
</section>
