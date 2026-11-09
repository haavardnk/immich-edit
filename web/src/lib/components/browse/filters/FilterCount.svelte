<script lang="ts">
  let { count }: { count: number } = $props();
</script>

{#if count > 0}
  <span
    class="grid h-4.5 min-w-4.5 place-items-center rounded-full bg-primary/20 px-1.25 text-[11px] font-semibold tracking-normal text-primary"
  >
    {count}<span class="sr-only">&nbsp;active</span>
  </span>
{/if}
