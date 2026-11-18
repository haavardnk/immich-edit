<script lang="ts">
  import { presetThumbUrl, type Preset } from '#lib/api/presets.ts';
  import SearchField from '#lib/components/SearchField.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiCheck, mdiCircleOffOutline, mdiMagnify } from '@mdi/js';
  import { onDestroy, tick } from 'svelte';
  import PresetActions from './PresetActions.svelte';
  import PresetAppliedCard from './PresetAppliedCard.svelte';
  import PresetGroups from './PresetGroups.svelte';
  import PresetTile from './PresetTile.svelte';
  import SavePresetDialog from './SavePresetDialog.svelte';

  let dialog = $state<{ preset?: Preset } | null>(null);
  let searching = $state(false);
  let field = $state<HTMLInputElement | null>(null);

  const applied = $derived(presetStrip.live);

  $effect(presetStrip.ensureLoaded);

  onDestroy(presetStrip.endPreview);

  async function openSearch(): Promise<void> {
    searching = true;
    await tick();
    field?.focus();
  }

  function closeSearch(): void {
    searching = false;
    presetStrip.query = '';
  }
</script>

<section aria-label="Presets" class="flex flex-col gap-2 pb-3">
  <div class="flex min-h-11 flex-none items-center gap-1 pe-2 pt-1">
    {#if searching}
      <div class="min-w-0 flex-1 ps-3">
        <SearchField bind:value={presetStrip.query} bind:ref={field} label="Search presets" />
      </div>
      <Button
        size="small"
        variant="ghost"
        color="primary"
        onmousedown={(event: MouseEvent) => event.preventDefault()}
        onclick={closeSearch}>Cancel</Button
      >
    {:else}
      <div class="min-w-0 flex-1">
        {#if presets.presets.length > 0}
          <PresetGroups chips />
        {/if}
      </div>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiMagnify}
        title=""
        aria-label="Search presets"
        onclick={() => void openSearch()}
      />
      <PresetActions oncreate={() => (dialog = {})} />
    {/if}
  </div>
  {#if applied}
    <PresetAppliedCard {applied} />
  {/if}
  <ul class="grid grid-cols-3 gap-x-2 gap-y-2.5 px-3 [[data-arrangement=inspector]_&]:grid-cols-4">
    {#if !presetStrip.searching}
      <li class="flex min-w-0 flex-col gap-1.5">
        <button
          type="button"
          class="relative grid aspect-square w-full place-items-center rounded-lg border bg-ghost text-dark/65 outline-none focus-visible:ring-2 focus-visible:ring-primary {applied
            ? 'border-hairline'
            : 'border-primary ring-1 ring-primary'}"
          aria-label="No preset"
          aria-current={applied ? undefined : true}
          disabled={!applied}
          onclick={presetStrip.remove}
        >
          <Icon icon={mdiCircleOffOutline} size="24px" aria-hidden="true" />
          {#if !applied}
            <span
              class="absolute right-1.5 bottom-1.5 grid size-5 place-items-center rounded-full bg-primary text-light"
              aria-hidden="true"
            >
              <Icon icon={mdiCheck} size="14px" />
            </span>
          {/if}
        </button>
        <span class="truncate text-center text-xs {applied ? 'text-dark/80' : 'text-light-950'}"
          >None</span
        >
      </li>
    {/if}
    {#each presetStrip.shown as preset (preset.id)}
      <PresetTile
        {preset}
        src={editor.assetId && editor.initialised
          ? presetThumbUrl(editor.assetId, preset, editor.savedHash)
          : null}
        width={0}
        disabled={!editor.initialised}
        onedit={() => (dialog = { preset })}
      />
    {/each}
  </ul>
  {#if presets.presets.length === 0}
    <p class="px-4 text-xs text-dark/65">No presets yet</p>
  {:else if presetStrip.shown.length === 0}
    <p class="px-4 text-xs text-dark/65">No presets match</p>
  {/if}
</section>

{#if dialog}
  <SavePresetDialog preset={dialog.preset} onClose={() => (dialog = null)} />
{/if}
