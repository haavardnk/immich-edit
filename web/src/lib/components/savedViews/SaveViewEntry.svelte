<script lang="ts">
  import { currentViewName, currentViewUrl } from '#lib/browse/savedViewActions.ts';
  import SaveViewForm from '#lib/components/savedViews/SaveViewForm.svelte';
  import { savedViews } from '#lib/stores/savedViews.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';

  let { id, title, onClose }: { id: string; title: string; onClose: (hadFocus: boolean) => void } =
    $props();

  let root = $state<HTMLElement | null>(null);

  function close(): void {
    onClose(root?.contains(document.activeElement) ?? false);
  }

  async function save(name: string): Promise<void> {
    const url = currentViewUrl();
    if (!url) {
      toasts.push('error', 'Too many filters to save this view');
      return;
    }
    if (await savedViews.create({ name, url })) close();
  }
</script>

<div bind:this={root} {id} class="border-b border-hairline px-3 py-3 touch:px-4">
  <SaveViewForm
    initial={currentViewName(title)}
    label="View name"
    onSubmit={save}
    onCancel={close}
  />
</div>
