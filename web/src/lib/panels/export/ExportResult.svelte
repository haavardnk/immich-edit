<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import type { ExportResult } from '#lib/stores/editor/export.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiAlertOutline, mdiRefresh } from '@mdi/js';

  let {
    result,
    busy,
    onRetry
  }: {
    result: ExportResult | null;
    busy: boolean;
    onRetry: () => void;
  } = $props();
</script>

{#if result}
  <Notice
    color={result.kind === 'success'
      ? 'success'
      : result.kind === 'duplicate'
        ? 'warning'
        : 'danger'}
    message={result.message}
  >
    {#if result.kind === 'error'}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-6 panel-action"
        leadingIcon={mdiRefresh}
        onclick={onRetry}
        disabled={busy}
      >
        Retry
      </Button>
    {/if}
  </Notice>
{/if}

{#if result && result.warnings.length > 0}
  <ul
    class="space-y-0.5 rounded-sm border border-warning-500/25 bg-warning-500/10 px-2 py-1.5 text-[10px] leading-snug text-warning-800"
  >
    {#each result.warnings as w (w)}
      <li class="flex items-start gap-1.5">
        <Icon icon={mdiAlertOutline} size="12px" class="mt-0.5 shrink-0" aria-hidden="true" />
        <span>{w}</span>
      </li>
    {/each}
  </ul>
{/if}
