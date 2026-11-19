<script lang="ts">
  import { listLuts, type LutMeta } from '#lib/api/luts.ts';
  import type { Preset } from '#lib/api/presets.ts';
  import HighlightText from '#lib/components/HighlightText.svelte';
  import SheetScrim from '#lib/components/sheet/SheetScrim.svelte';
  import { commandLabel, isEnabled, isShown, runCommand } from '#lib/commands/command.ts';
  import { EDITOR_PALETTE_MENU } from '#lib/commands/menus.ts';
  import type { IconCommand } from '#lib/commands/types.ts';
  import { commitLut, commitPanelReset } from '#lib/components/editor/controls/paramReset.ts';
  import { defaultApplyOptions } from '#lib/edits/applyPreset.ts';
  import { DESKTOP_PANELS } from '#lib/editor/params/catalog.ts';
  import {
    searchSettings,
    settingValue,
    textMatch,
    type Highlight
  } from '#lib/editor/settingsSearch.ts';
  import { revealDevelopSetting } from '#lib/panels/developPanels.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { isKeybind } from '#lib/shortcuts/match.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { holdModalSheet } from '#lib/stores/overlays.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
  import { Icon } from '@immich/ui';
  import {
    mdiCubeOutline,
    mdiMagnify,
    mdiPinOutline,
    mdiRestore,
    mdiTuneVariant,
    mdiViewGridOutline
  } from '@mdi/js';
  import { Command, Dialog } from 'bits-ui';

  type Action = { id: string; label: string; icon: string; run: () => void };

  const SETTING_LIMIT = 8;
  const LIBRARY_LIMIT = 5;
  const HEADING = 'px-3 pt-2.5 pb-1 text-[10px] font-semibold tracking-wide text-dark/65 uppercase';
  const ROW =
    'flex h-9 cursor-pointer items-center gap-2.5 px-3 text-[13px] text-dark outline-none data-selected:bg-primary/12';

  const SETTING_VALUE = 'setting:';

  let query = $state('');
  let highlighted = $state('');
  let luts = $state<LutMeta[]>([]);
  let pending: (() => void) | null = null;
  const open = $derived(settingSearch.paletteOpen);

  const commandActions = $derived<Action[]>(
    EDITOR_PALETTE_MENU.flat()
      .filter((command) => isShown(command) && isEnabled(command))
      .map((command: IconCommand) => ({
        id: command.id,
        label: commandLabel(command),
        icon: command.icon,
        run: () => runCommand(command)
      }))
  );
  const resetActions: Action[] = DESKTOP_PANELS.map((panel) => ({
    id: `reset:${panel.id}`,
    label: `Reset ${panel.title}`,
    icon: mdiRestore,
    run: () => commitPanelReset(panel)
  }));

  const searching = $derived(query.trim() !== '');
  const settings = $derived(searching ? searchSettings(query).slice(0, SETTING_LIMIT) : []);
  const presetMatches = $derived(named(presets.presets, (preset) => preset.name));
  const lutMatches = $derived(named(luts, (lut) => lut.name));
  const actions = $derived(
    named([...commandActions, ...(searching ? resetActions : [])], (action) => action.label)
  );

  $effect(() => {
    if (open) return holdModalSheet();
  });

  $effect(() => {
    if (open && !presets.loaded && !presets.loading) void presets.load();
  });

  $effect(() => {
    if (!open) return;
    query = '';
    listLuts()
      .then((list) => (luts = list))
      .catch(() => (luts = []));
  });

  function named<T>(
    items: readonly T[],
    name: (item: T) => string
  ): { item: T; at: Highlight | null }[] {
    if (!searching) return items.map((item) => ({ item, at: null }));
    return items
      .map((item) => ({ item, at: textMatch(name(item), query) }))
      .filter((entry) => entry.at !== null);
  }

  function setOpen(next: boolean): void {
    settingSearch.paletteOpen = next;
    if (!next) editor.endPresetPreview();
  }

  function choose(run: () => void): void {
    pending = run;
    setOpen(false);
  }

  function afterClose(event: Event): void {
    const run = pending;
    pending = null;
    if (!run) return;
    event.preventDefault();
    run();
  }

  function toggleKey(event: KeyboardEvent): void {
    if (!isKeybind(event, 'searchSettings')) return;
    event.preventDefault();
    event.stopPropagation();
    setOpen(false);
  }

  function pinKey(event: KeyboardEvent): void {
    if (!isKeybind(event, 'pinSetting') || !highlighted.startsWith(SETTING_VALUE)) return;
    const id = highlighted.slice(SETTING_VALUE.length);
    if (!settings.some(({ setting }) => setting.id === id && setting.param)) return;
    event.preventDefault();
    event.stopPropagation();
    void preferences.togglePin(id);
  }

  function previewPreset(preset: Preset): void {
    editor.previewPreset({ manifest: preset.manifest, opts: defaultApplyOptions() });
  }

  function applyPreset(preset: Preset): void {
    void editor.applyPreset(preset.manifest, defaultApplyOptions(), preset.name);
  }
</script>

<Dialog.Root bind:open={() => open, setOpen}>
  <Dialog.Portal>
    <SheetScrim tone="light" />
    <Dialog.Content
      class="fixed top-[12%] left-1/2 z-50 flex max-h-[70%] w-[min(40rem,calc(100%-2rem))] -translate-x-1/2 flex-col overflow-hidden rounded-xl border border-hairline bg-editor-panel shadow-2xl"
      onCloseAutoFocus={afterClose}
      onkeydown={toggleKey}
    >
      <Dialog.Title class="sr-only">Search settings and actions</Dialog.Title>
      <Command.Root
        bind:value={highlighted}
        shouldFilter={false}
        loop
        class="flex min-h-0 flex-col"
        onkeydown={pinKey}
      >
        <div class="flex shrink-0 items-center gap-2 border-b border-hairline px-3">
          <Icon icon={mdiMagnify} size="18px" class="text-dark/65" aria-hidden="true" />
          <Command.Input
            bind:value={query}
            aria-label="Search settings and actions"
            placeholder="Search settings, presets and actions"
            class="h-11 min-w-0 flex-1 bg-transparent text-sm text-light-950 caret-primary outline-none placeholder:text-dark/50"
          />
        </div>
        <Command.List class="min-h-0 flex-1 overflow-y-auto pb-1.5 scrollbar-hidden">
          <Command.Empty class="px-3 py-6 text-center text-xs text-dark/65"
            >No matches</Command.Empty
          >
          {#if settings.length > 0}
            <Command.Group>
              <Command.GroupHeading class={HEADING}>Settings</Command.GroupHeading>
              <Command.GroupItems>
                {#each settings as { setting, highlight } (setting.id)}
                  <Command.Item
                    value="{SETTING_VALUE}{setting.id}"
                    class={ROW}
                    onSelect={() => choose(() => revealDevelopSetting(setting))}
                  >
                    <Icon icon={mdiTuneVariant} size="15px" class="shrink-0 text-dark/65" />
                    <span class="shrink-0 text-light-950">
                      <HighlightText text={setting.label} range={highlight} />
                    </span>
                    <span class="min-w-0 flex-1 truncate text-xs text-dark/65"
                      >{setting.desktopPath}</span
                    >
                    {#if preferences.isPinned(setting.id)}
                      <Icon
                        icon={mdiPinOutline}
                        size="13px"
                        class="shrink-0 text-primary"
                        aria-label="Pinned"
                      />
                    {/if}
                    <span class="shrink-0 font-mono text-[11px] text-dark/65 tabular-nums">
                      {settingValue(setting, editor.edits, editor.isRaw)}
                    </span>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          {/if}
          {#if searching && presetMatches.length + lutMatches.length > 0}
            <Command.Group>
              <Command.GroupHeading class={HEADING}>Presets and LUTs</Command.GroupHeading>
              <Command.GroupItems>
                {#each presetMatches.slice(0, LIBRARY_LIMIT) as { item: preset, at } (preset.id)}
                  <Command.Item
                    value="preset:{preset.id}"
                    class={ROW}
                    onSelect={() => choose(() => applyPreset(preset))}
                    onpointerenter={() => previewPreset(preset)}
                    onpointerleave={editor.endPresetPreview}
                  >
                    <Icon icon={mdiViewGridOutline} size="15px" class="shrink-0 text-dark/65" />
                    <span class="min-w-0 truncate text-light-950">
                      <HighlightText text={preset.name} range={at} />
                    </span>
                    <span class="min-w-0 flex-1 truncate text-xs text-dark/65">
                      Preset{preset.group_name ? ` · ${preset.group_name}` : ''}
                    </span>
                  </Command.Item>
                {/each}
                {#each lutMatches.slice(0, LIBRARY_LIMIT) as { item: lut, at } (lut.id)}
                  <Command.Item
                    value="lut:{lut.id}"
                    class={ROW}
                    onSelect={() => choose(() => commitLut(lut.id))}
                  >
                    <Icon icon={mdiCubeOutline} size="15px" class="shrink-0 text-dark/65" />
                    <span class="min-w-0 truncate text-light-950">
                      <HighlightText text={lut.name} range={at} />
                    </span>
                    <span class="min-w-0 flex-1 truncate text-xs text-dark/65">LUT</span>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          {/if}
          {#if actions.length > 0}
            <Command.Group>
              <Command.GroupHeading class={HEADING}>Actions</Command.GroupHeading>
              <Command.GroupItems>
                {#each actions as { item: action, at } (action.id)}
                  <Command.Item
                    value="action:{action.id}"
                    class={ROW}
                    onSelect={() => choose(action.run)}
                  >
                    <Icon icon={action.icon} size="15px" class="shrink-0 text-dark/65" />
                    <span class="min-w-0 flex-1 truncate text-light-950">
                      <HighlightText text={action.label} range={at} />
                    </span>
                  </Command.Item>
                {/each}
              </Command.GroupItems>
            </Command.Group>
          {/if}
        </Command.List>
        <div
          class="flex shrink-0 items-center gap-4 border-t border-hairline px-3 py-1.5 text-[11px] text-dark/65"
          aria-hidden="true"
        >
          <span><kbd class="font-mono">↑↓</kbd> move</span>
          <span><kbd class="font-mono">↵</kbd> open</span>
          {#if highlighted.startsWith(SETTING_VALUE)}
            <span><kbd class="font-mono">{keysFor('pinSetting')}</kbd> pin</span>
          {/if}
          <span><kbd class="font-mono">esc</kbd> close</span>
        </div>
      </Command.Root>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
