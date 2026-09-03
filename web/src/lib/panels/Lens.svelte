<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { editsToManifest } from '$lib/edits/manifest';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_LENS } from '$lib/types/edits';
  import { Button } from '@immich/ui';

  const profile = $derived(editor.lensProfile);
  const profileError = $derived(editor.lensProfileError);
  const profileLoading = $derived(profile === null && profileError === null);

  const hasProfile = $derived(!!profile?.edits);
  const hasCa = $derived(
    !!profile?.edits &&
      (profile.edits.ca_red_scale_x10000 !== 0 || profile.edits.ca_blue_scale_x10000 !== 0)
  );
  const isRaw = $derived(editor.meta?.is_raw ?? false);
  const autoOn = $derived(isRaw && hasProfile);
  const profileEnabled = $derived(editor.edits.lens.profile_enabled ?? autoOn);
  const isAuto = $derived(editor.edits.lens.profile_enabled === null && autoOn);
  const modified = $derived('lens_profile' in editsToManifest(editor.edits).ops);

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

  function onToggleProfile(enabled: boolean): void {
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
    void editor.onCommit('Lens Profile');
  }

  function onToggleCa(enabled: boolean): void {
    editor.edits.lens.ca_enabled = enabled;
    if (
      enabled &&
      editor.edits.lens.ca_red_scale_x10000 === 0 &&
      editor.edits.lens.ca_blue_scale_x10000 === 0
    ) {
      loadProfileCoefficients();
    }
    void editor.onCommit('Chromatic Aberration');
  }

  function onToggleConstrain(checked: boolean): void {
    if (editor.edits.lens.profile_enabled === null) {
      editor.edits.lens.profile_enabled = autoOn;
      loadProfileCoefficients();
    }
    editor.edits.lens.constrain_crop = checked;
    void editor.onCommit('Constrain Crop');
  }

  function resetLens(): void {
    editor.edits.lens = { ...NEUTRAL_LENS };
    void editor.onCommit('Reset Lens Corrections');
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex flex-col gap-1">
    <SectionHeader
      title="Lens Profile"
      {modified}
      onReset={resetLens}
      resetTitle="Reset Lens Corrections"
    />
    {#if profileLoading}
      <div class="text-[11px] text-dark/65">Loading…</div>
    {:else if profileError}
      <Notice message={profileError}>
        <Button size="tiny" variant="ghost" color="secondary" onclick={editor.retryLensProfile}>
          Try again
        </Button>
      </Notice>
    {:else if profile?.matched && profile.lens}
      <div class="text-[11px] leading-tight text-dark/65">
        {profile.lens}
        {#if profile.focal_length}
          <span class="text-dark/65">
            · {profile.focal_length.toFixed(0)}mm{#if profile.aperture}
              · f/{profile.aperture.toFixed(1)}{/if}
          </span>
        {/if}
      </div>
    {:else}
      <div class="text-[11px] text-dark/65">No matching lens profile</div>
    {/if}
  </div>

  <CheckboxRow
    label="Enable Profile Corrections"
    checked={profileEnabled}
    disabled={!hasProfile}
    onChange={onToggleProfile}
  >
    {#if isAuto}
      <span class="text-dark/65"> · Auto</span>
    {/if}
  </CheckboxRow>

  <CheckboxRow
    label="Remove Chromatic Aberration"
    checked={editor.edits.lens.ca_enabled}
    disabled={!hasCa}
    onChange={onToggleCa}
  />

  <EditSlider
    label="Distortion"
    commitAction="Lens Distortion"
    bind:value={editor.edits.lens.distortion_amount}
    min={0}
    max={200}
    defaultValue={100}
    disabled={!profileEnabled}
  />

  <EditSlider
    label="Vignetting"
    commitAction="Lens Vignetting"
    bind:value={editor.edits.lens.vignette_amount}
    min={0}
    max={200}
    defaultValue={100}
    disabled={!profileEnabled}
  />

  <CheckboxRow
    label="Constrain Crop"
    checked={editor.lensView.constrain_crop}
    disabled={!profileEnabled}
    onChange={onToggleConstrain}
  />
</div>
