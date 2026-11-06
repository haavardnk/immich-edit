<script lang="ts">
  import FilterToggle from '#lib/components/browse/FilterToggle.svelte';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  const scope = bulkScope;
  const actions = bulkActions;
</script>

<IconButton
  size="medium"
  variant="ghost"
  color="secondary"
  shape="round"
  class="size-11"
  icon={mdiClose}
  title={hint(scope.closeLabel, 'gridClearSelection')}
  aria-label={scope.closeLabel}
  disabled={scope.selectingAll}
  onclick={actions.clear}
/>
<p
  class="min-w-0 flex-1 ps-1 text-lg font-semibold whitespace-nowrap text-light-950"
  aria-live="polite"
>
  {scope.targetCount} selected
</p>
{#if scope.canSelectAll}
  <Button
    size="small"
    variant="ghost"
    color="primary"
    class="h-11 shrink-0 px-3 text-[15px] font-semibold"
    loading={scope.selectingAll}
    disabled={actions.busy || scope.loadingMore}
    onclick={() => void actions.selectAll()}
  >
    Select all
  </Button>
{/if}
<FilterToggle class="hidden wide:block" />
