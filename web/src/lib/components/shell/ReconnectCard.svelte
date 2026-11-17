<script lang="ts">
  import Wordmark from '#lib/components/Wordmark.svelte';
  import type { Reconnect } from '#lib/connection/reconnect.svelte.ts';
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    cause,
    reconnect,
    actions
  }: { title: string; cause?: string; reconnect: Reconnect; actions?: Snippet } = $props();
</script>

<div class="auth-stage flex h-full w-full items-center justify-center px-5 py-8">
  <main class="w-full max-w-xs">
    <Wordmark bright class="mb-6" />
    <h1 class="text-xl font-semibold text-light-950">{title}</h1>
    {#if cause}
      <p class="mt-1.5 text-sm text-dark/80">{cause}</p>
    {/if}
    <div class="mt-5 flex gap-2 [&>*]:flex-1">
      <Button
        type="button"
        size="small"
        color="primary"
        loading={reconnect.probing}
        disabled={reconnect.probing}
        onclick={() => void reconnect.retryNow()}
      >
        Retry now
      </Button>
      {@render actions?.()}
    </div>
    {#if !reconnect.offline}
      <p
        class="mt-3 text-center text-xs text-dark/80 tabular-nums"
        data-testid="reconnect-countdown"
      >
        {reconnect.probing ? 'Retrying…' : `Retrying in ${reconnect.secondsLeft} s`}
      </p>
    {/if}
  </main>
</div>
