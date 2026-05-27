<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';

  function reload(): void {
    location.reload();
  }

  function home(): void {
    void goto('/photos');
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-immich-dark-bg text-immich-dark-fg p-6">
  <div class="max-w-md w-full bg-immich-dark-gray border border-white/10 rounded-xl p-6 space-y-4">
    <h1 class="text-lg font-medium">Something went wrong</h1>
    <p class="text-sm text-immich-dark-fg/70">{page.error?.message ?? 'Unexpected error'}</p>
    {#if page.status}
      <p class="text-xs text-immich-dark-fg/40">Status {page.status}</p>
    {/if}
    <div class="flex gap-2">
      <button class="px-3 py-1.5 rounded bg-white/10 hover:bg-white/20 text-sm" onclick={reload}>Reload</button>
      <button class="px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 text-sm" onclick={home}>Go to Photos</button>
    </div>
  </div>
</div>
