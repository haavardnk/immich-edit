<script lang="ts">
  let { text, range }: { text: string; range: readonly [number, number] | null } = $props();

  const start = $derived(range?.[0] ?? text.length);
  const end = $derived(range?.[1] ?? text.length);
</script>

<span
  >{text.slice(0, start)}<mark class="bg-transparent text-primary">{text.slice(start, end)}</mark
  >{text.slice(end)}</span
>
