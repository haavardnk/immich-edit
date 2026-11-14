<script lang="ts">
  import { goto } from '$app/navigation';
  import Dialog from '#lib/components/Dialog.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiPlus } from '@mdi/js';

  let open = $state(false);
  let name = $state('');
  let creating = $state(false);
  let nameInput = $state<HTMLInputElement | null>(null);
  let generation = 0;

  $effect(() => nameInput?.focus());

  function show(): void {
    generation += 1;
    name = '';
    open = true;
  }

  function close(): void {
    generation += 1;
    open = false;
  }

  async function create(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed || creating) return;
    creating = true;
    const current = generation;
    try {
      const album = await library.createAlbum(trimmed);
      if (!album || current !== generation) return;
      close();
      await goto(`/albums/${album.id}`);
    } finally {
      creating = false;
    }
  }
</script>

<IconButton
  size="small"
  variant="ghost"
  color="secondary"
  class="touch:size-11"
  icon={mdiPlus}
  title="Create album"
  aria-label="Create album"
  onclick={show}
/>

{#if open}
  <Dialog title="Create album" onClose={close}>
    <form class="flex flex-col gap-4" onsubmit={(e) => void create(e)}>
      <TextInput
        label="Name"
        bind:ref={nameInput}
        bind:value={name}
        autocomplete="off"
        class="touch:[&_input]:h-11 touch:[&_input]:text-[15px]"
      />
      <div class="flex justify-end gap-2">
        <Button
          type="button"
          size="small"
          variant="ghost"
          color="secondary"
          class="touch:h-11"
          onclick={close}>Cancel</Button
        >
        <Button
          type="submit"
          size="small"
          class="touch:h-11"
          loading={creating}
          disabled={!name.trim() || creating}>Create</Button
        >
      </div>
    </form>
  </Dialog>
{/if}
