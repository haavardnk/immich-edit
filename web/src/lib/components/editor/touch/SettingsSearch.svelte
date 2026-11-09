<script lang="ts">
  import HighlightText from '#lib/components/HighlightText.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import SearchField from '#lib/components/SearchField.svelte';
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import { revealSetting } from '#lib/editor/revealSetting.ts';
  import {
    byTouchHome,
    modifiedSettings,
    settingValue,
    type Setting
  } from '#lib/editor/settingsSearch.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { Button } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  let input = $state<HTMLInputElement | null>(null);

  const modified = $derived(modifiedSettings(editor.edits, editor.isRaw));
  const modifiedIds = $derived(new Set(modified.map((setting) => setting.id)));
  const live = $derived(ui.developModifiedOnly && !settingSearch.filtering);
  const results = $derived(
    ui.developModifiedOnly
      ? settingSearch.matches.filter((match) => modifiedIds.has(match.setting.id))
      : settingSearch.matches
  );
  const chip =
    'flex h-8 items-center rounded-full px-3.5 text-[13.5px] outline-none focus-visible:ring-2 focus-visible:ring-primary data-[state=checked]:bg-primary/22 data-[state=checked]:text-light-950 data-[state=unchecked]:bg-ghost data-[state=unchecked]:text-dark';

  $effect(() => {
    input?.focus();
  });

  function close(): void {
    settingSearch.clear();
    touchUi.closeSearch();
  }

  function jump(setting: Setting): void {
    const { group, view } = setting.touch;
    close();
    touchUi.setGroup(group);
    touchUi.setView(view);
    touchUi.raise();
    void revealSetting(setting.id, { fallback: `#touch-view-${group}-${view}` });
  }
</script>

<div class="flex shrink-0 items-center gap-2 px-3 pt-1.5">
  <div class="min-w-0 flex-1">
    <SearchField bind:ref={input} bind:value={settingSearch.query} label="Search settings" />
  </div>
  <Button size="small" variant="ghost" color="primary" class="shrink-0 px-2" onclick={close}>
    {settingSearch.filtering ? 'Cancel' : 'Done'}
  </Button>
</div>
<RadioGroup.Root
  bind:value={
    () => (ui.developModifiedOnly ? 'modified' : 'all'),
    (value) => (ui.developModifiedOnly = value === 'modified')
  }
  orientation="horizontal"
  aria-label="Settings shown"
  class="flex shrink-0 gap-1.5 px-3 py-2"
>
  <RadioGroup.Item value="all" class={chip}>All</RadioGroup.Item>
  <RadioGroup.Item value="modified" class={chip}>Modified · {modified.length}</RadioGroup.Item>
</RadioGroup.Root>
<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain pb-2 scrollbar-hidden">
  {#if live}
    {#each byTouchHome(modified) as group (group.home.path)}
      <section aria-label={group.home.path} class="flex flex-col px-3 pb-1">
        <h3 class="px-1 pt-2 pb-0.5 text-[11px] font-semibold tracking-wide text-dark/65 uppercase">
          {group.home.path}
        </h3>
        {#each group.settings as setting (setting.id)}
          {#if setting.param}
            <ParamSlider param={setting.param} />
          {:else}
            <MenuRow
              class="-mx-3 w-auto"
              label={setting.label}
              aria-label="Open {setting.label}"
              onclick={() => jump(setting)}
            />
          {/if}
        {/each}
      </section>
    {:else}
      <p class="px-4 py-8 text-center text-sm text-dark/65">No modified settings</p>
    {/each}
  {:else}
    <ul aria-label="Settings" class="flex flex-col">
      {#each results as { setting, highlight } (setting.id)}
        <li>
          <MenuRow
            class="h-auto! py-1.5"
            aria-label="{setting.label}, {setting.touch.path}"
            onclick={() => jump(setting)}
          >
            <span class="flex min-w-0 flex-1 flex-col items-start gap-0.5">
              <span class="max-w-full truncate text-[15px] leading-5 text-light-950">
                <HighlightText text={setting.label} range={highlight} />
              </span>
              <span class="max-w-full truncate text-xs leading-4 text-dark/65"
                >{setting.touch.path}</span
              >
            </span>
            <span class="shrink-0 font-mono text-xs text-dark tabular-nums">
              {settingValue(setting, editor.edits, editor.isRaw)}
            </span>
          </MenuRow>
        </li>
      {:else}
        <li class="px-4 py-8 text-center text-sm text-dark/65">No matching settings</li>
      {/each}
    </ul>
  {/if}
</div>
