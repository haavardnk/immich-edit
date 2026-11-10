<script lang="ts">
  let { text, class: className = '' }: { text: string; class?: string } = $props();
</script>

<span
  data-compare-label
  class="pointer-events-none absolute top-2.5 max-w-[45%] truncate rounded-full bg-black/55 px-2 py-0.5 text-[11px] font-medium text-white {className}"
  >{text}</span
>
