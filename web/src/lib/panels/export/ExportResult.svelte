<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import ExportWarnings from '#lib/panels/export/ExportWarnings.svelte';
  import type { ExportResult } from '#lib/stores/editor/export.svelte.ts';
  import { Button } from '@immich/ui';
  import { mdiRefresh } from '@mdi/js';

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
  <ExportWarnings warnings={result.warnings} />
{/if}
