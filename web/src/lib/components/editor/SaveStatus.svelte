<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { mdiCheck, mdiAlertCircleOutline, mdiLoading } from '@mdi/js';
</script>

{#if editor.asset}
  {#if editor.saveError}
    <button
      class="flex items-center gap-1 text-[11px] text-error hover:text-error/80 transition-colors"
      title={editor.saveError}
      onclick={() => void editor.retrySave()}
    >
      <Icon path={mdiAlertCircleOutline} size={12} />
      Save failed — retry
    </button>
  {:else if editor.saving}
    <span class="flex items-center gap-1 text-[11px] text-immich-dark-fg/60">
      <Icon path={mdiLoading} size={12} class="animate-spin" />
      Saving…
    </span>
  {:else}
    <span class="flex items-center gap-1 text-[11px] text-immich-dark-fg/40">
      <Icon path={mdiCheck} size={12} />
      Saved
    </span>
  {/if}
{/if}
