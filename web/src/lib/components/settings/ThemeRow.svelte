<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import AccentPicker from '#lib/components/settings/AccentPicker.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import ThemePicker from '#lib/components/settings/ThemePicker.svelte';
  import ThemeSwatch from '#lib/components/settings/ThemeSwatch.svelte';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { themeEntry } from '#lib/theme/registry.ts';
  import { resolveTheme } from '#lib/theme/resolve.ts';
  import type { Scheme } from '#lib/theme/types.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronRight } from '@mdi/js';

  let { scheme }: { scheme: Scheme } = $props();

  let open = $state(false);

  const label = $derived(scheme === 'dark' ? 'Dark theme' : 'Light theme');
  const accentLabel = $derived(scheme === 'dark' ? 'Dark accent' : 'Light accent');
  const entry = $derived(
    themeEntry(
      scheme === 'dark' ? appearance.current.darkTheme : appearance.current.lightTheme,
      scheme
    )
  );
  const resolved = $derived(
    resolveTheme(entry.family, entry.theme, appearance.current.accents[scheme])
  );
</script>

<SettingsRow {label} anchor="{scheme}-theme">
  <div class="touch:hidden">
    <AccentPicker {scheme} label={accentLabel} compact />
  </div>
  <Popover
    {open}
    onOpenChange={(next) => (open = next)}
    align="end"
    appearance="panel"
    contentClass="w-[36rem] p-3"
    sheetClass="px-4 pt-1 pb-4"
    title={label}
  >
    {#snippet trigger(props)}
      <Button
        {...props}
        size="small"
        variant="ghost"
        color="secondary"
        class="shrink-0 gap-2 bg-light-200/60 ps-1.5 pe-2 font-normal touch:static touch:bg-transparent touch:px-0 touch:after:absolute touch:after:inset-0"
        aria-label="{label}: {entry.theme.name}"
      >
        <ThemeSwatch palette={resolved.palette} accent={resolved.accentColor} size="chip" />
        <span class="text-[13px] text-light-950 touch:hidden">{entry.theme.name}</span>
        <Icon
          icon={mdiChevronDown}
          size="14px"
          class="text-dark/80 touch:hidden"
          aria-hidden="true"
        />
        <Icon
          icon={mdiChevronRight}
          size="20px"
          class="hidden text-dark/45 touch:block"
          aria-hidden="true"
        />
      </Button>
    {/snippet}
    <div class="flex flex-col gap-4">
      <ThemePicker {scheme} {label} bare />
      <div class="hidden touch:block">
        <AccentPicker {scheme} label={accentLabel} />
      </div>
    </div>
  </Popover>
</SettingsRow>
