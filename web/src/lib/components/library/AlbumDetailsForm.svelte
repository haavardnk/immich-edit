<script lang="ts">
  import {
    ALBUM_DESCRIPTION_MAX,
    albumDetailsPatch,
    validAlbumDetails
  } from '#lib/browse/albumDetails.ts';
  import FormDialog from '#lib/components/FormDialog.svelte';
  import TextArea from '#lib/components/TextArea.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { albumActions, type AlbumDetailsRequest } from '#lib/stores/albumActions.svelte.ts';
  import { charCount } from '#lib/utils/charCount.ts';
  import { Field } from '@immich/ui';
  import { untrack } from 'svelte';

  let { request }: { request: AlbumDetailsRequest } = $props();

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
</script>

<FormDialog
  title={target ? 'Album details' : 'Create album'}
  submitLabel={target ? 'Save' : 'Create'}
  saving={albumActions.saving}
  disabled={!valid || !changed}
  onSubmit={() => void albumActions.save(name, description)}
  onClose={albumActions.closeDetails}
>
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
</FormDialog>
