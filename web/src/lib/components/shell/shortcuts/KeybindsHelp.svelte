<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import TextInput from '#lib/components/TextInput.svelte';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { activeContexts } from '#lib/shortcuts/context.ts';
  import { customCount } from '#lib/shortcuts/custom.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { isKeybind } from '#lib/shortcuts/match.ts';
  import type { KeybindContext, KeybindId } from '#lib/shortcuts/table.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon, Kbd } from '@immich/ui';
  import { mdiAccountOutline, mdiMagnify, mdiRestore } from '@mdi/js';
  import { helpSections, type HelpGroup } from './helpSections';
  import { ShortcutRecorder } from './recorder.svelte.ts';
  import ShortcutRow from './ShortcutRow.svelte';

  const CONTEXT_LABELS: Record<KeybindContext, string> = {
    global: 'Anywhere',
    grid: 'the grid',
    loupe: 'the loupe',
    compare: 'compare',
    survey: 'survey',
    editor: 'the editor',
    geometry: 'the Geometry panel',
    masks: 'the Masks panel',
    retouch: 'the Retouch panel',
    viewer: 'the viewer window'
  };

  const recorder = new ShortcutRecorder();

  let query = $state('');
  let input = $state<HTMLInputElement | null>(null);

  const contexts = $derived(ui.keybindsHelpOpen ? activeContexts() : []);
  const hereLabel = $derived(contexts[0] ? CONTEXT_LABELS[contexts[0]] : 'here');
  const custom = $derived(customCount(preferences.shortcuts));
  const sections = $derived(helpSections(contexts, query, (bind) => keysFor(bind.id as KeybindId)));

  function onWindowKeydown(e: KeyboardEvent): void {
    if (!ui.keybindsHelpOpen) return;
    if (recorder.keydown(e)) {
      e.stopPropagation();
      return;
    }
    if (isKeybind(e, 'editorEscape') || (isKeybind(e, 'help') && e.target !== input)) {
      e.preventDefault();
      ui.closeKeybindsHelp();
    }
    e.stopPropagation();
  }

  function resetAll(): void {
    recorder.cancel();
    void preferences.setShortcuts({});
  }

  function focusableScroll(node: HTMLElement): void {
    node.tabIndex = 0;
  }

  $effect(() => {
    if (!ui.keybindsHelpOpen) return;
    query = '';
    recorder.cancel();
    input?.focus();
  });
</script>

<svelte:window onkeydowncapture={onWindowKeydown} />

{#if ui.keybindsHelpOpen}
  <Dialog
    title="Keyboard shortcuts"
    size="giant"
    onClose={ui.closeKeybindsHelp}
    bodyClass="flex flex-col overflow-hidden p-0!"
  >
    {#snippet actions()}
      <div class="flex-1 min-w-0">
        <TextInput
          bind:ref={input}
          bind:value={query}
          type="text"
          size="tiny"
          placeholder="Filter shortcuts"
          aria-label="Filter shortcuts"
          leadingIcon={searchIcon}
          class="bg-light-200! ring-transparent! focus-within:bg-light-200! focus-within:ring-primary!"
        />
      </div>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        leadingIcon={mdiRestore}
        class="shrink-0"
        disabled={custom === 0}
        onclick={resetAll}
      >
        Reset all
      </Button>
    {/snippet}

    {#snippet footer()}
      <div class="flex w-full flex-wrap items-center justify-between gap-x-4 gap-y-1 text-xs">
        <span class="flex min-w-0 items-center gap-1.5 text-dark/65">
          <Icon icon={mdiAccountOutline} size="14px" class="shrink-0" aria-hidden="true" />
          {countOf(custom, 'custom shortcut')} · saved to your account and used on every device with a
          keyboard
        </span>
        <span class="flex items-center gap-1 text-dark/65">
          Click a key to change it · <Kbd size="tiny">{keyLabel('Escape')}</Kbd> cancels
        </span>
      </div>
    {/snippet}

    <div
      role="region"
      aria-label="Keyboard shortcuts"
      use:focusableScroll
      class="immich-scrollbar min-h-0 flex-1 overflow-y-auto px-5 py-5 touch:max-h-[calc(var(--app-height)*0.7)]"
    >
      {#if sections.here.length === 0 && sections.elsewhere.length === 0}
        <p class="text-xs text-dark/65">No shortcuts match “{query}”.</p>
      {:else}
        {#if sections.here.length > 0}
          <h3 class="text-[11px] uppercase tracking-wider text-primary mb-2">
            Available in {hereLabel}
          </h3>
          {@render groupList(sections.here)}
        {/if}

        {#if sections.elsewhere.length > 0}
          <h3
            class="text-[11px] uppercase tracking-wider text-dark/65 mb-2 {sections.here.length > 0
              ? 'mt-3 pt-3 border-t border-dark/10'
              : ''}"
          >
            Elsewhere in the app
          </h3>
          {@render groupList(sections.elsewhere)}
        {/if}
      {/if}
    </div>
  </Dialog>
{/if}

{#snippet searchIcon(_disabled: boolean)}
  <Icon icon={mdiMagnify} size="60%" aria-hidden />
{/snippet}

{#snippet groupList(groups: HelpGroup[])}
  <div class="columns-1 sm:columns-2 gap-8">
    {#each groups as group (group.title)}
      <div class="break-inside-avoid mb-5">
        <h4 class="text-[11px] uppercase tracking-wider text-dark/65 mb-1.5">
          {group.title}
        </h4>
        <div class="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1 text-xs">
          {#each group.binds as bind (bind.id)}
            <ShortcutRow {bind} {recorder} />
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}
