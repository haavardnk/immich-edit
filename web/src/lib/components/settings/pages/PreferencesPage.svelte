<script lang="ts">
  import InstallRow from '#lib/components/settings/InstallRow.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import ThemeModeChoice from '#lib/components/settings/ThemeModeChoice.svelte';
  import ThemeRow from '#lib/components/settings/ThemeRow.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { LAYOUT_CHOICE_LABELS, LAYOUT_CHOICES } from '#lib/device/layoutRules.ts';
  import { layoutHint } from '#lib/device/status.ts';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import {
    RENDERER_CHOICE_LABELS,
    renderer,
    type RendererChoice
  } from '#lib/stores/renderer.svelte.ts';
  import { rendererStatus } from '#lib/stores/rendererStatus.ts';
  import type { Scheme } from '#lib/theme/types.ts';
  import { Link, Switch } from '@immich/ui';

  const LOCAL_NETWORK_DOCS =
    'https://haavardnk.github.io/immich-edit/rendering/#browser-previews-on-a-local-network';

  const SCHEME_LABELS: Readonly<Record<Scheme, string>> = { light: 'Light', dark: 'Dark' };

  const LAYOUT_OPTIONS = LAYOUT_CHOICES.map((value) => ({
    value,
    label: LAYOUT_CHOICE_LABELS[value]
  }));

  const RENDERER_OPTIONS = (
    ['auto', 'browser', 'server'] as const satisfies readonly RendererChoice[]
  ).map((value) => ({ value, label: RENDERER_CHOICE_LABELS[value] }));

  const active = $derived(appearance.scheme);
  const schemes = $derived<Scheme[]>(active === 'dark' ? ['dark', 'light'] : ['light', 'dark']);
  const modeHint = $derived(
    appearance.current.mode === 'system' ? `${SCHEME_LABELS[active]} in use` : null
  );
  const layoutStatus = $derived(layoutHint(layout.choice, layout.mode));
  const rendererHint = $derived(renderer.choice === 'server' ? null : rendererStatus(renderer));
  const insecureBrowser = $derived(renderer.choice !== 'server' && !isSecureContext);
</script>

{#snippet localNetwork()}
  <Link href={LOCAL_NETWORK_DOCS} class="text-primary">
    Enable browser previews on a local network
  </Link>
{/snippet}

<div class="flex flex-col gap-7 touch:gap-5">
  <SettingsGroup title="Appearance" note="Every device" anchor="appearance">
    <SettingsRow label="Mode" hint={modeHint} anchor="mode" stack>
      <ThemeModeChoice label="Mode" compact fill="touch" />
    </SettingsRow>
    {#each schemes as scheme (scheme)}
      <ThemeRow {scheme} />
    {/each}
  </SettingsGroup>

  <SettingsGroup title="This device" anchor="device">
    <SettingsRow label="Layout" hint={layoutStatus} anchor="layout" stack>
      <SegmentedChoice
        options={LAYOUT_OPTIONS}
        value={layout.choice}
        label="Layout"
        appearance="view"
        fill="touch"
        onChange={(choice) => layout.setChoice(choice)}
      />
    </SettingsRow>
    <SettingsRow
      label="Preview renderer"
      hint={rendererHint}
      anchor="renderer"
      stack
      detail={insecureBrowser ? localNetwork : undefined}
    >
      <SegmentedChoice
        options={RENDERER_OPTIONS}
        value={renderer.choice}
        label="Preview renderer"
        appearance="view"
        fill="touch"
        onChange={(choice) => renderer.setChoice(choice)}
      />
    </SettingsRow>
    <SettingsRow label="Save exports automatically" anchor="save-automatically">
      <Switch
        checked={exportSettings.saveAutomatically}
        aria-label="Save exports automatically"
        onCheckedChange={(checked) => (exportSettings.saveAutomatically = checked)}
      />
    </SettingsRow>
    <InstallRow />
  </SettingsGroup>
</div>
