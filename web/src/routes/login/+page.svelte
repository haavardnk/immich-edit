<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { login } from '$lib/api/auth';
  import { ApiError } from '$lib/api/client';

  let token = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (submitting || !token) return;
    submitting = true;
    error = null;
    try {
      await login(token);
      const next = page.url.searchParams.get('next') ?? '/';
      const safe = next.startsWith('/') && !next.startsWith('//') ? next : '/';
      await goto(safe, { replaceState: true, invalidateAll: true });
    } catch (e: unknown) {
      if (e instanceof ApiError && e.status === 401) {
        error = 'Invalid token';
      } else {
        error = (e as Error)?.message ?? 'Login failed';
      }
      submitting = false;
    }
  }
</script>

<div class="h-full w-full flex items-center justify-center p-6">
  <form
    onsubmit={submit}
    class="w-full max-w-sm flex flex-col gap-4 p-6 rounded-lg bg-immich-dark-gray border border-immich-dark-gray"
  >
    <h1 class="text-xl font-semibold">immich-edit</h1>
    <label class="flex flex-col gap-1">
      <span class="text-sm opacity-70">Access token</span>
      <input
        type="password"
        autocomplete="current-password"
        bind:value={token}
        disabled={submitting}
        class="px-3 py-2 rounded bg-black/30 border border-white/10 focus:outline-none focus:ring-1 focus:ring-immich-primary"
      />
    </label>
    {#if error}
      <p class="text-sm text-red-400">{error}</p>
    {/if}
    <button
      type="submit"
      disabled={submitting || !token}
      class="px-3 py-2 rounded bg-immich-primary text-white disabled:opacity-50"
    >
      {submitting ? 'Signing in…' : 'Sign in'}
    </button>
  </form>
</div>
