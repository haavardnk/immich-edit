<script lang="ts">
  import { browseTarget, setDescription } from '#lib/browse/assetMeta.ts';
  import { keyHeld } from '#lib/input/modifiers.svelte.ts';
  import { ownsAsset } from '#lib/browse/ownership.ts';
  import InfoSection from '#lib/components/browse/info/InfoSection.svelte';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Textarea } from '@immich/ui';
  import { untrack } from 'svelte';

  type Draft = { id: string; text: string };

  let { asset }: { asset: AssetSummary } = $props();

  const saved = $derived(asset.exifInfo?.description ?? '');
  const owned = $derived(ownsAsset(asset));

  let edit = $state<Draft | null>(null);
  let sent: Draft | null = null;
  let field = $state<HTMLTextAreaElement | null>(null);

  const draft = $derived(edit?.id === asset.id ? edit.text : saved);

  async function commit(pending: Draft | null): Promise<void> {
    if (!pending || pending === sent) return;
    sent = pending;
    await setDescription(browseTarget(pending.id), pending.text.trim());
    if (edit === pending) edit = null;
  }

  $effect(() => {
    const id = asset.id;
    return () => {
      untrack(() => {
        if (edit?.id === id) void commit(edit);
      });
    };
  });

  function oninput(event: Event & { currentTarget: HTMLTextAreaElement }): void {
    edit = { id: asset.id, text: event.currentTarget.value };
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !keyHeld('lineBreak', event) && !event.isComposing) {
      event.preventDefault();
      field?.blur();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      edit = null;
      field?.blur();
    }
  }
</script>

{#if owned || saved !== ''}
  <InfoSection title="Description">
    {#if owned}
      <Textarea
        bind:ref={field}
        value={draft}
        size="small"
        rows={2}
        grow
        aria-label="Description"
        placeholder="Add a description"
        class="max-h-40 bg-light-950/6! text-[13px] ring-hairline! touch:text-[15px]"
        {oninput}
        onblur={() => void commit(edit)}
        {onkeydown}
      />
    {:else}
      <p class="text-[13px] whitespace-pre-line text-light-950/90 touch:text-[15px]">{saved}</p>
    {/if}
  </InfoSection>
{/if}
