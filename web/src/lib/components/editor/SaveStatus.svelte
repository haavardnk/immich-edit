<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiAlertCircleOutline, mdiCheck, mdiLoading } from '@mdi/js';
</script>

{#if editor.asset}
  {#if editor.saveError}
    <Button
      size="tiny"
      variant="ghost"
      color="danger"
      class="gap-1 p-0 text-[11px]"
      title={editor.saveError}
      onclick={() => void editor.retrySave()}
    >
      <Icon icon={mdiAlertCircleOutline} size="12px" aria-hidden="true" />
      Save failed — retry
    </Button>
  {:else if editor.saving}
    <span class="flex items-center gap-1 text-[11px] text-dark/65" role="status">
      <Icon icon={mdiLoading} size="12px" class="animate-spin" aria-hidden="true" />
      Saving…
    </span>
  {:else}
    <span class="flex items-center gap-1 text-[11px] text-dark/65" role="status">
      <Icon icon={mdiCheck} size="12px" aria-hidden="true" />
      Saved
    </span>
  {/if}
{/if}
