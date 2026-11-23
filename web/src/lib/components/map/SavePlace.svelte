<script lang="ts">
  import type { GeoLocation } from '#lib/browse/locationOps.ts';
  import Popover from '#lib/components/Popover.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { places } from '#lib/stores/places.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiStar, mdiStarOutline } from '@mdi/js';

  let { at, suggested }: { at: GeoLocation; suggested: string } = $props();

  let open = $state(false);
  let name = $state('');
  let busy = $state(false);

  const saved = $derived(places.savedAt(at));
  const title = $derived(saved ? 'Saved place' : 'Save place');

  function setOpen(next: boolean): void {
    if (next) name = saved?.name ?? suggested;
    open = next;
  }

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed || busy) return;
    busy = true;
    const done = saved
      ? await places.rename(saved, trimmed)
      : (await places.create(trimmed, at)) !== null;
    busy = false;
    if (done) open = false;
  }

  function remove(): void {
    if (!saved) return;
    open = false;
    void places.remove(saved);
  }
</script>

<Popover
  {open}
  onOpenChange={setOpen}
  anchor="top"
  align="end"
  appearance="panel"
  {title}
  contentClass="w-72 p-3"
  tooltip={saved ? `Saved as ${saved.name}` : 'Save this place'}
>
  {#snippet trigger(props)}
    <IconButton
      {...props}
      size="small"
      variant="ghost"
      color={saved ? 'primary' : 'secondary'}
      class="shrink-0 touch:size-11"
      icon={saved ? mdiStar : mdiStarOutline}
      title=""
      aria-label={title}
      aria-pressed={saved !== null}
    />
  {/snippet}
  <form class="flex flex-col gap-3" onsubmit={submit}>
    <TextInput label="Name" bind:value={name} maxlength={80} />
    <p class="text-caption text-dark/65">Shown in search, the Location menu and on the map</p>
    <div class="flex items-center justify-end gap-2">
      {#if saved}
        <Button
          type="button"
          size="small"
          variant="ghost"
          color="danger"
          class="me-auto touch:h-11"
          onclick={remove}
        >
          Remove
        </Button>
      {:else}
        <Button
          type="button"
          size="small"
          variant="ghost"
          color="secondary"
          class="touch:h-11"
          onclick={() => (open = false)}
        >
          Cancel
        </Button>
      {/if}
      <Button
        type="submit"
        size="small"
        class="touch:h-11"
        loading={busy}
        disabled={name.trim() === '' || busy}
      >
        {saved ? 'Rename' : 'Save place'}
      </Button>
    </div>
  </form>
</Popover>
