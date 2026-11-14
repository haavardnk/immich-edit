<script lang="ts">
  let { url, alt }: { url: string; alt: string } = $props();

  let loaded = $state(false);
</script>

<img
  src={url}
  {alt}
  draggable="false"
  data-testid="version-hold"
  class="pointer-events-none absolute inset-0 z-10 size-full object-contain select-none {loaded
    ? 'bg-image-canvas'
    : 'opacity-0'}"
  onload={() => (loaded = true)}
/>
