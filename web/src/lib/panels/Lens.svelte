<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_LENS } from '$lib/types/edits';
  import { getLensProfile, type LensProfileMatch } from '$lib/api/lensProfile';

  let profile = $state<LensProfileMatch | null>(null);
  let profileLoading = $state(false);
  let profileError = $state<string | null>(null);
  let lastFetchedAssetId = $state<string | null>(null);

  $effect(() => {
    const id = editor.assetId;
    if (!id || id === lastFetchedAssetId) return;
    lastFetchedAssetId = id;
    profile = null;
    profileError = null;
    profileLoading = true;
    getLensProfile(id)
      .then((p) => {
        profile = p;
      })
      .catch((e: unknown) => {
        profileError = e instanceof Error ? e.message : String(e);
      })
      .finally(() => {
        profileLoading = false;
      });
  });

  const hasProfile = $derived(!!profile?.edits);
  const hasCa = $derived(
    !!profile?.edits &&
      (profile.edits.ca_red_scale_x10000 !== 0 || profile.edits.ca_blue_scale_x10000 !== 0)
  );

  function loadProfileCoefficients(): void {
    const e = profile?.edits;
    if (!e) return;
    const l = editor.edits.lens;
    l.k1 = e.k1;
    l.k2 = e.k2;
    l.k3 = e.k3;
    l.vk1 = e.vk1;
    l.vk2 = e.vk2;
    l.vk3 = e.vk3;
    l.ca_red_scale_x10000 = e.ca_red_scale_x10000;
    l.ca_blue_scale_x10000 = e.ca_blue_scale_x10000;
  }

  function onToggleProfile(e: Event): void {
    const enabled = (e.currentTarget as HTMLInputElement).checked;
    editor.edits.lens.profile_enabled = enabled;
    if (enabled) {
      if (editor.edits.lens.k1 === 0 && editor.edits.lens.k2 === 0 && editor.edits.lens.k3 === 0) {
        loadProfileCoefficients();
      }
      if (editor.edits.lens.distortion_amount === 0)
        editor.edits.lens.distortion_amount = NEUTRAL_LENS.distortion_amount;
      if (editor.edits.lens.vignette_amount === 0)
        editor.edits.lens.vignette_amount = NEUTRAL_LENS.vignette_amount;
    }
    editor.onCommit('Lens Profile');
  }

  function onToggleCa(e: Event): void {
    const enabled = (e.currentTarget as HTMLInputElement).checked;
    editor.edits.lens.ca_enabled = enabled;
    if (
      enabled &&
      editor.edits.lens.ca_red_scale_x10000 === 0 &&
      editor.edits.lens.ca_blue_scale_x10000 === 0
    ) {
      loadProfileCoefficients();
    }
    editor.onCommit('Chromatic Aberration');
  }

  function onToggleConstrain(e: Event): void {
    editor.edits.lens.constrain_crop = (e.currentTarget as HTMLInputElement).checked;
    editor.onCommit('Constrain Crop');
  }
</script>

<div class="flex flex-col gap-3">
  <div class="flex flex-col gap-1">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Lens Profile</div>
    {#if profileLoading}
      <div class="text-[11px] text-immich-dark-fg/50">Loading…</div>
    {:else if profileError}
      <div class="text-[11px] text-red-400/80">{profileError}</div>
    {:else if profile?.matched && profile.lens}
      <div class="text-[11px] text-immich-dark-fg/70 leading-tight">
        {profile.lens}
        {#if profile.focal_length}
          <span class="text-immich-dark-fg/40">
            · {profile.focal_length.toFixed(0)}mm{#if profile.aperture}
              · f/{profile.aperture.toFixed(1)}{/if}
          </span>
        {/if}
      </div>
    {:else}
      <div class="text-[11px] text-immich-dark-fg/50">No matching lens profile</div>
    {/if}
  </div>

  <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
    <input
      type="checkbox"
      class="checkbox checkbox-xs checkbox-primary"
      checked={editor.edits.lens.profile_enabled}
      disabled={!hasProfile}
      onchange={onToggleProfile}
    />
    Enable Profile Corrections
  </label>

  <label class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer">
    <input
      type="checkbox"
      class="checkbox checkbox-xs checkbox-primary"
      checked={editor.edits.lens.ca_enabled}
      disabled={!hasCa}
      onchange={onToggleCa}
    />
    Remove Chromatic Aberration
  </label>

  <SliderRow
    label="Distortion"
    commitAction="Lens Distortion"
    bind:value={editor.edits.lens.distortion_amount}
    min={0}
    max={200}
    step={1}
    defaultValue={100}
    disabled={!editor.edits.lens.profile_enabled}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />

  <SliderRow
    label="Vignetting"
    commitAction="Lens Vignetting"
    bind:value={editor.edits.lens.vignette_amount}
    min={0}
    max={200}
    step={1}
    defaultValue={100}
    disabled={!editor.edits.lens.profile_enabled}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />

  <label
    class="flex items-center gap-2 text-[11px] text-immich-dark-fg/80 cursor-pointer"
    class:opacity-50={!editor.edits.lens.profile_enabled}
  >
    <input
      type="checkbox"
      class="checkbox checkbox-xs checkbox-primary"
      checked={editor.edits.lens.constrain_crop}
      disabled={!editor.edits.lens.profile_enabled}
      onchange={onToggleConstrain}
    />
    Constrain Crop
  </label>
</div>
