<script lang="ts">
  import { getMapSettings, setMapSettings, type MapSettings } from '#lib/api/map.ts';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { mapConfig } from '#lib/map/mapConfig.svelte.ts';
  import {
    draftOf,
    draftProblem,
    PHOTON_PUBLIC,
    sameSettings,
    settingsOf,
    type MapSettingsDraft,
    type StyleMode
  } from '#lib/map/mapSettingsForm.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { errorMessage } from '#lib/utils/errors.ts';
  import { Button } from '@immich/ui';
  import { onMount } from 'svelte';

  const MODES = [
    { value: 'immich', label: 'Follow Immich' },
    { value: 'custom', label: 'Custom' }
  ] as const satisfies readonly { value: StyleMode; label: string }[];

  let saved = $state<MapSettings | null>(null);
  let draft = $state<MapSettingsDraft | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  const problem = $derived(draft ? draftProblem(draft) : null);
  const dirty = $derived(
    saved !== null &&
      draft !== null &&
      (draft.mode !== draftOf(saved).mode || !sameSettings(saved, settingsOf(draft)))
  );

  async function load(): Promise<void> {
    try {
      saved = await getMapSettings();
      draft = draftOf(saved);
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function save(): Promise<void> {
    if (!draft || busy || problem) return;
    busy = true;
    error = null;
    try {
      saved = await setMapSettings(settingsOf(draft));
      draft = draftOf(saved);
      mapConfig.reset();
      toasts.push('success', 'Map settings saved');
    } catch (e) {
      error = errorMessage(e);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    void load();
  });
</script>

<SettingsGroup title="Map" anchor="map-settings">
  {#snippet action()}
    {#if dirty}
      <Button
        size="tiny"
        loading={busy}
        disabled={busy || problem !== null}
        onclick={() => void save()}
      >
        Save
      </Button>
    {/if}
  {/snippet}
  {#if draft}
    <SettingsRow
      label="Map style"
      hint={error ?? problem ?? 'Map tiles load from this style’s server in each viewer’s browser'}
      anchor="map-style"
      stack
    >
      <SegmentedChoice
        options={MODES}
        value={draft.mode}
        label="Map style"
        onChange={(mode) => {
          if (draft) draft.mode = mode;
        }}
      />
    </SettingsRow>
    {#if draft.mode === 'custom'}
      <div class="flex flex-col gap-2 px-4 py-3">
        <TextInput
          class="font-mono"
          bind:value={draft.light}
          placeholder="https://tiles.openfreemap.org/styles/positron"
          aria-label="Light style URL"
          label="Light style URL"
        />
        <TextInput
          class="font-mono"
          bind:value={draft.dark}
          placeholder="https://tiles.openfreemap.org/styles/dark"
          aria-label="Dark style URL"
          label="Dark style URL"
        />
      </div>
    {/if}
    <SettingsRow
      label="Place search"
      hint="Searches go to this Photon server through immich-edit. Empty turns search off."
      anchor="place-search"
      stack
    >
      <div class="w-72 shrink-0 touch:w-full">
        <TextInput
          class="font-mono"
          bind:value={draft.geocoder}
          placeholder={PHOTON_PUBLIC}
          aria-label="Photon server URL"
        />
      </div>
    </SettingsRow>
  {:else if error}
    <SettingsRow label="Map" hint={error} />
  {/if}
</SettingsGroup>
