<script lang="ts">
  import SaveViewButton from '#lib/components/savedViews/SaveViewButton.svelte';
  import SaveViewEntry from '#lib/components/savedViews/SaveViewEntry.svelte';
  import { tick, type Snippet } from 'svelte';

  let { title, children }: { title: string; children: Snippet<[Snippet<[boolean]>, Snippet]> } =
    $props();

  const formId = $props.id();

  let saving = $state(false);
  let button: HTMLElement | null = null;

  async function close(hadFocus: boolean): Promise<void> {
    saving = false;
    if (!hadFocus) return;
    await tick();
    button?.focus();
  }
</script>

{#snippet toggle(iconOnly: boolean)}
  <SaveViewButton
    {iconOnly}
    open={saving}
    controls={formId}
    onRef={(node) => (button = node)}
    onclick={() => (saving = !saving)}
  />
{/snippet}

{#snippet entry()}
  {#if saving}
    <SaveViewEntry id={formId} {title} onClose={(hadFocus) => void close(hadFocus)} />
  {/if}
{/snippet}

{@render children(toggle, entry)}
