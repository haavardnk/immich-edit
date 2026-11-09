<script lang="ts">
  import type { Snippet } from 'svelte';

  let { label, children }: { label?: string; children: Snippet } = $props();
</script>

<div
  class="grid grid-cols-[3.875rem_minmax(0,1fr)] items-start gap-x-2 touch:grid-cols-[4.75rem_minmax(0,1fr)]"
>
  <span
    class="flex h-6.5 items-center text-xs text-dark/65 select-none touch:h-10 touch:text-[13px]"
    aria-hidden="true"
  >
    {label ?? ''}
  </span>
  <div class="min-w-0">{@render children()}</div>
</div>
