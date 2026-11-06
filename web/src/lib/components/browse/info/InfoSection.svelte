<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, action, children }: { title?: string; action?: Snippet; children: Snippet } =
    $props();

  const headingId = $props.id();
</script>

<section
  aria-labelledby={title ? headingId : undefined}
  class="flex flex-col gap-2 px-4 py-3 touch:gap-2.5"
>
  {#if title}
    <div class="flex min-h-5 items-center justify-between gap-2">
      <h3
        id={headingId}
        class="text-[10px] font-medium tracking-[.06em] text-light-950/60 uppercase touch:text-[11px]"
      >
        {title}
      </h3>
      {@render action?.()}
    </div>
  {/if}
  {@render children()}
</section>
