<script lang="ts">
  import { ui } from '$lib/stores/ui.svelte';

  const shortcuts: { keys: string; description: string }[] = [
    { keys: '← / →', description: 'Previous / next asset' },
    { keys: 'Space', description: 'Toggle zoom (fit ↔ 200%)' },
    { keys: '0', description: 'Fit zoom to viewport' },
    { keys: '1 – 5', description: 'Set / clear star rating' },
    { keys: 'F', description: 'Toggle favorite' },
    { keys: '⇧F', description: 'Toggle fullscreen' },
    { keys: '\\ (hold)', description: 'Show original' },
    { keys: '⌘Z / ⌘⇧Z', description: 'Undo / redo' },
    { keys: 'Esc', description: 'Close help or exit fullscreen' },
    { keys: '?', description: 'Toggle this help' },
  ];

  function onBackdropClick(e: MouseEvent): void {
    if (e.currentTarget === e.target) ui.closeKeybindsHelp();
  }
</script>

{#if ui.keybindsHelpOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={onBackdropClick}
  >
    <div
      class="bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl p-5 min-w-[320px] max-w-[420px]"
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
      <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
        {#each shortcuts as s (s.keys)}
          <kbd class="font-mono text-immich-dark-fg/90 whitespace-nowrap">{s.keys}</kbd>
          <span class="text-immich-dark-fg/70">{s.description}</span>
        {/each}
      </div>
    </div>
  </div>
{/if}
