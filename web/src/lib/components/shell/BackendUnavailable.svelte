<script lang="ts">
  import { live } from '$lib/api/health';
  import Logo from '$lib/components/Logo.svelte';

  let { retry }: { retry: () => void } = $props();

  const DELAYS = [1000, 2000, 4000, 8000, 10000];

  let attempt = $state(0);
  let probing = $state(false);

  async function probe(): Promise<boolean> {
    probing = true;
    try {
      await live();
      return true;
    } catch {
      return false;
    } finally {
      probing = false;
    }
  }

  async function attemptNow(): Promise<void> {
    if (probing) return;
    if (await probe()) {
      attempt = 0;
      retry();
    } else {
      attempt += 1;
    }
  }

  $effect(() => {
    const delay = DELAYS[Math.min(attempt, DELAYS.length - 1)];
    const timer = setTimeout(() => void attemptNow(), delay);
    return () => clearTimeout(timer);
  });
</script>

<div class="h-full w-full flex items-center justify-center p-6">
  <div
    class="w-full max-w-md flex flex-col gap-4 p-6 rounded-lg bg-immich-dark-gray border border-immich-dark-gray"
  >
    <h1 class="flex items-center gap-2 text-xl font-semibold tracking-tight">
      <Logo size={26} />
      <span
        ><span class="text-immich-dark-fg/90">immich</span><span style="color:#6366F1">-edit</span
        ></span
      >
    </h1>
    <p class="text-sm font-medium">Can't reach the immich-edit server.</p>
    <p class="text-sm opacity-70">
      It may still be starting up, or it may not be running. This page reconnects automatically.
    </p>
    <button
      type="button"
      class="rounded bg-immich-primary px-3 py-2 text-sm font-medium text-black disabled:opacity-60"
      disabled={probing}
      onclick={() => void attemptNow()}
    >
      {probing ? 'Reconnecting…' : 'Retry now'}
    </button>
  </div>
</div>
