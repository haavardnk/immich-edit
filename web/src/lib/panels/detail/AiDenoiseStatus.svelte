<script lang="ts">
  import type { DenoiseStatus } from '$lib/api/denoise';
  import ModelMissingNotice from '$lib/components/ModelMissingNotice.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { Button, ProgressBar } from '@immich/ui';

  let {
    status,
    onRetry
  }: {
    status: DenoiseStatus | null;
    onRetry: () => void;
  } = $props();

  const percent = $derived(
    status?.state === 'running' && status.total > 0
      ? Math.round((status.done / status.total) * 100)
      : null
  );
</script>

{#if status?.state === 'queued' || status?.state === 'running'}
  <div class="flex min-h-6 flex-col justify-center gap-1 text-[11px] text-dark/65">
    <div class="flex items-center justify-between gap-2">
      <span role="status">
        {status.state === 'queued' ? 'Waiting to denoise…' : 'Denoising at full resolution…'}
      </span>
      {#if percent !== null}
        <span class="tabular-nums" aria-hidden="true">{percent}%</span>
      {/if}
    </div>
    {#if status.state === 'running'}
      <ProgressBar
        progress={percent ?? 0}
        max={100}
        valueLabel={percent === null ? undefined : `${percent}%`}
        size="tiny"
        shape="round"
        color="primary"
        stop={false}
        aria-label="AI Denoise progress"
      />
    {/if}
  </div>
{:else if status?.state === 'model_missing'}
  <ModelMissingNotice
    message="The AI Denoise model is not installed."
    askAdmin="Ask an administrator to install it."
  />
{:else if status?.state === 'disabled'}
  <Notice color="warning" message="AI models are turned off on this server." />
{:else if status?.state === 'unsupported'}
  <Notice color="info" message="AI Denoise needs a raw file with usable color data." />
{:else if status?.state === 'failed'}
  <Notice message={`AI Denoise failed: ${status.error}`}>
    <Button size="tiny" variant="ghost" color="secondary" onclick={onRetry}>Try again</Button>
  </Notice>
{/if}
