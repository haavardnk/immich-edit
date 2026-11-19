<script lang="ts">
  let {
    lead,
    label,
    tail = '',
    mono = false,
    class: className = ''
  }: { lead: string; label: string; tail?: string; mono?: boolean; class?: string } = $props();
</script>

<span
  role="status"
  data-photo-pill
  class="pointer-events-none absolute top-3 left-1/2 z-20 max-w-[calc(100%-1.5rem)] -translate-x-1/2 truncate rounded-full bg-black/70 px-3 py-1 text-xs whitespace-nowrap text-white shadow-lg backdrop-blur-sm {className}"
  >{lead}
  <span class="font-semibold text-image-accent {mono ? 'font-mono' : ''}">{label}</span>{tail
    ? ` ${tail}`
    : ''}</span
>
