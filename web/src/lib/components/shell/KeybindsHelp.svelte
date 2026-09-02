<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { KEYBINDS, isKeybind, keysFor, type KeybindContext } from '$lib/keybinds';
  import { activeContexts } from '$lib/keybindContext';
  import Dialog from '$lib/components/Dialog.svelte';
  import { Icon } from '@immich/ui';
  import { mdiMagnify } from '@mdi/js';

  type HelpBind = { id: string; keys: string; label: string };
  type HelpGroup = { title: string; binds: HelpBind[] };

  const CONTEXT_LABELS: Record<KeybindContext, string> = {
    global: 'Anywhere',
    grid: 'the grid',
    loupe: 'the loupe',
    compare: 'compare',
    survey: 'survey',
    editor: 'the editor',
    masks: 'the Masks panel',
    retouch: 'the Retouch panel'
  };

  let query = $state('');
  let input = $state<HTMLInputElement | null>(null);

  const contexts = $derived(ui.keybindsHelpOpen ? activeContexts() : []);
  const hereLabel = $derived(contexts[0] ? CONTEXT_LABELS[contexts[0]] : 'here');

  const sections = $derived.by<{ here: HelpGroup[]; elsewhere: HelpGroup[] }>(() => {
    const needle = query.trim().toLowerCase();
    const here = new Map<string, HelpGroup>();
    const elsewhere = new Map<string, HelpGroup>();
    for (const bind of KEYBINDS) {
      const keys = keysFor(bind.id);
      if (needle && !`${bind.label} ${keys}`.toLowerCase().includes(needle)) continue;
      const active = bind.contexts.some((c: KeybindContext) => contexts.includes(c));
      const target = active ? here : elsewhere;
      const group = target.get(bind.group) ?? { title: bind.group, binds: [] };
      group.binds.push({ id: bind.id, keys, label: bind.label });
      target.set(bind.group, group);
    }
    return { here: [...here.values()], elsewhere: [...elsewhere.values()] };
  });

  function onWindowKeydown(e: KeyboardEvent): void {
    if (!ui.keybindsHelpOpen) return;
    if (isKeybind(e, 'editorEscape') || (isKeybind(e, 'help') && e.target !== input)) {
      e.preventDefault();
      ui.closeKeybindsHelp();
    }
    e.stopPropagation();
  }

  function focusableScroll(node: HTMLElement): void {
    node.tabIndex = 0;
  }

  $effect(() => {
    if (!ui.keybindsHelpOpen) return;
    query = '';
    input?.focus();
  });
</script>

<svelte:window onkeydowncapture={onWindowKeydown} />

{#if ui.keybindsHelpOpen}
  <Dialog title="Keyboard shortcuts" size="giant" onClose={ui.closeKeybindsHelp}>
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
          class="bg-neutral-800! ring-transparent! focus-within:bg-neutral-800! focus-within:ring-primary!"
        />
      </div>
    {/snippet}

    <div
      role="region"
      aria-label="Keyboard shortcuts"
      use:focusableScroll
      class="max-h-[calc(100dvh-8rem)] overflow-y-auto"
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
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
          {#each group.binds as bind (bind.id)}
            <kbd class="font-mono text-dark/90 whitespace-nowrap">{bind.keys}</kbd>
            <span class="text-dark/70">{bind.label}</span>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}
