<script lang="ts">
  import { page } from '$app/state';
  import { ui } from '$lib/stores/ui.svelte';
  import { browseView } from '$lib/stores/browseView.svelte';
  import { KEYBIND_GROUPS, type KeybindMode } from '$lib/keybinds';

  const mode = $derived<KeybindMode>(
    page.url.pathname.startsWith('/assets/')
      ? 'editor'
      : browseView.loupeId
        ? 'loupe'
        : 'grid'
  );
  const groups = $derived(KEYBIND_GROUPS.filter((g) => g.mode === mode));

  function onBackdropClick(e: MouseEvent): void {
    if (e.currentTarget === e.target) ui.closeKeybindsHelp();
  }

  function onWindowKeydown(e: KeyboardEvent): void {
    if (!ui.keybindsHelpOpen) return;
    if (e.key === 'Escape' || e.key === '?' || (e.key === '/' && e.shiftKey)) {
      e.preventDefault();
      ui.closeKeybindsHelp();
    }
    e.stopPropagation();
  }
</script>

<svelte:window onkeydowncapture={onWindowKeydown} />

{#if ui.keybindsHelpOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={onBackdropClick}
  >
    <div
      class="bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl p-5 min-w-[320px] max-w-105"
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard shortcuts"
    >
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-sm font-medium text-immich-dark-fg">Keyboard shortcuts</h2>
        <button
          class="text-xs px-2 py-0.5 rounded text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
          onclick={ui.closeKeybindsHelp}
          aria-label="close"
        >
          Esc
        </button>
      </div>
      <div class="flex flex-col gap-4">
        {#each groups as group (group.title)}
          <div>
            <h3 class="text-[11px] uppercase tracking-wider text-immich-dark-fg/40 mb-1.5">
              {group.title}
            </h3>
            <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
              {#each group.binds as s (s.keys + s.description)}
                <kbd class="font-mono text-immich-dark-fg/90 whitespace-nowrap">{s.keys}</kbd>
                <span class="text-immich-dark-fg/70">{s.description}</span>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
