<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';
  import { KEYBINDS, hint, isKeybind, keysFor, type KeybindContext } from '$lib/keybinds';
  import { activeContexts } from '$lib/keybindContext';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiClose, mdiMagnify } from '@mdi/js';

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

  function onBackdropClick(e: MouseEvent): void {
    if (e.currentTarget === e.target) ui.closeKeybindsHelp();
  }

  function onWindowKeydown(e: KeyboardEvent): void {
    if (!ui.keybindsHelpOpen) return;
    if (isKeybind(e, 'editorEscape') || (isKeybind(e, 'help') && e.target !== input)) {
      e.preventDefault();
      ui.closeKeybindsHelp();
    }
    e.stopPropagation();
  }

  $effect(() => {
    if (!ui.keybindsHelpOpen) return;
    query = '';
    input?.focus();
  });
</script>

<svelte:window onkeydowncapture={onWindowKeydown} />

{#if ui.keybindsHelpOpen}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/60 backdrop-blur-sm p-8"
    role="presentation"
    onclick={onBackdropClick}
  >
    <div
      class="bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl flex flex-col max-h-full w-full max-w-4xl"
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard shortcuts"
    >
      <div class="flex items-center gap-3 px-5 py-3 border-b border-white/10">
        <h2 class="text-sm font-medium text-immich-dark-fg flex-none">Keyboard shortcuts</h2>
        <div
          class="flex items-center gap-2 flex-1 min-w-0 bg-black/30 rounded-md px-2 py-1 border border-white/10 focus-within:border-immich-primary/60"
        >
          <Icon path={mdiMagnify} size={16} class="text-immich-dark-fg/40 flex-none" />
          <input
            bind:this={input}
            bind:value={query}
            type="text"
            placeholder="Filter shortcuts"
            aria-label="Filter shortcuts"
            class="flex-1 min-w-0 bg-transparent text-xs text-immich-dark-fg placeholder:text-immich-dark-fg/30 outline-none"
          />
        </div>
        <button
          class="flex-none p-1 rounded text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
          onclick={ui.closeKeybindsHelp}
          aria-label="close"
          title={hint('Close', 'editorEscape')}
        >
          <Icon path={mdiClose} size={16} />
        </button>
      </div>

      <div class="overflow-y-auto px-5 py-4">
        {#if sections.here.length === 0 && sections.elsewhere.length === 0}
          <p class="text-xs text-immich-dark-fg/50">No shortcuts match “{query}”.</p>
        {:else}
          {#if sections.here.length > 0}
            <h3 class="text-[11px] uppercase tracking-wider text-immich-primary mb-2">
              Available in {hereLabel}
            </h3>
            {@render groupList(sections.here)}
          {/if}

          {#if sections.elsewhere.length > 0}
            <h3
              class="text-[11px] uppercase tracking-wider text-immich-dark-fg/40 mb-2 {sections.here
                .length > 0
                ? 'mt-3 pt-3 border-t border-white/10'
                : ''}"
            >
              Elsewhere in the app
            </h3>
            <div class="opacity-50">
              {@render groupList(sections.elsewhere)}
            </div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

{#snippet groupList(groups: HelpGroup[])}
  <div class="columns-1 sm:columns-2 gap-8">
    {#each groups as group (group.title)}
      <div class="break-inside-avoid mb-5">
        <h4 class="text-[11px] uppercase tracking-wider text-immich-dark-fg/40 mb-1.5">
          {group.title}
        </h4>
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
          {#each group.binds as bind (bind.id)}
            <kbd class="font-mono text-immich-dark-fg/90 whitespace-nowrap">{bind.keys}</kbd>
            <span class="text-immich-dark-fg/70">{bind.label}</span>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}
