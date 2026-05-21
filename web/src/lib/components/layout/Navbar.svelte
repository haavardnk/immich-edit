<script lang="ts">
  let { title = 'immich-edit', back = null }: { title?: string; back?: string | null } = $props();
</script>

<nav class="navbar bg-base-200 shadow-md">
  <div class="flex-1 gap-2">
    {#if back}
      <a class="btn btn-sm btn-ghost" href={back}>← Back</a>
    {/if}
    <a class="btn btn-ghost text-xl normal-case" href="/">immich-edit</a>
    {#if title !== 'immich-edit'}
      <span class="text-sm opacity-60">/ {title}</span>
    {/if}
  </div>
</nav>
