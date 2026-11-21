<script lang="ts">
  import {
    ALBUM_DESCRIPTION_MAX,
    albumDetailsPatch,
    charCount,
    validAlbumDetails
  } from '#lib/browse/albumDetails.ts';
  import Dialog from '#lib/components/Dialog.svelte';
  import TextArea from '#lib/components/TextArea.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { albumActions, type AlbumDetailsRequest } from '#lib/stores/albumActions.svelte.ts';
  import { Button, Field } from '@immich/ui';
  import { untrack } from 'svelte';

  let { request }: { request: AlbumDetailsRequest } = $props();

  const FORM_ID = 'album-details';
  const HINT_ID = 'album-details-hint';

  const target = untrack(() => request.album);
  const focus = untrack(() => request.focus);

  let name = $state(target?.albumName ?? '');
  let description = $state(target?.description ?? '');
  let nameInput = $state<HTMLInputElement | null>(null);
  let descriptionInput = $state<HTMLTextAreaElement | null>(null);

  const used = $derived(charCount(description.trim()));
  const valid = $derived(validAlbumDetails(name, description));
  const changed = $derived(!target || albumDetailsPatch(target, name, description) !== null);

  $effect(() => {
    (focus === 'name' ? nameInput : descriptionInput)?.focus();
  });

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!valid || !changed) return;
    void albumActions.save(name, description);
  }
</script>

<Dialog title={target ? 'Album details' : 'Create album'} onClose={albumActions.closeDetails}>
  {#snippet footer()}
    <div class="flex w-full justify-end gap-2">
      <Button
        type="button"
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        onclick={albumActions.closeDetails}
      >
        Cancel
      </Button>
      <Button
        type="submit"
        form={FORM_ID}
        size="small"
        class="touch:h-11"
        loading={albumActions.saving}
        disabled={!valid || !changed || albumActions.saving}
      >
        {target ? 'Save' : 'Create'}
      </Button>
    </div>
  {/snippet}

  <form id={FORM_ID} class="flex flex-col gap-4" onsubmit={submit}>
    <TextInput
      label="Name"
      color="neutral"
      bind:ref={nameInput}
      bind:value={name}
      autocomplete="off"
      class="touch:[&_input]:h-11 touch:[&_input]:text-body"
    />
    <div class="flex flex-col gap-1">
      <Field label="Description" size="small">
        <TextArea
          bind:ref={descriptionInput}
          bind:value={description}
          rows={4}
          aria-describedby={HINT_ID}
          class="text-sm touch:text-body"
        />
      </Field>
      <p id={HINT_ID} class="flex justify-between gap-3 text-caption text-dark/65">
        <span>Shown in Immich too</span>
        <span class="tabular-nums {used > ALBUM_DESCRIPTION_MAX ? 'text-danger' : ''}">
          {used} / {ALBUM_DESCRIPTION_MAX}
        </span>
      </p>
    </div>
  </form>
</Dialog>
