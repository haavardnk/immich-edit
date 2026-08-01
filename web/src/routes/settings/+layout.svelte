<script lang="ts">
  import { page } from '$app/state';

  let { children }: { children: import('svelte').Snippet } = $props();

  const tabs = [
    { href: '/settings', label: 'Settings' },
    { href: '/settings/diagnostics', label: 'Diagnostics' }
  ];
</script>

<div class="flex-1 min-h-0 overflow-y-auto scrollbar-hidden p-6">
  <div class="max-w-3xl mx-auto space-y-6">
    <nav class="flex items-center gap-1 border-b border-white/10 pb-2 text-xs">
      {#each tabs as tab (tab.href)}
        <a
          href={tab.href}
          class="rounded px-2.5 py-1.5 transition-colors {page.url.pathname === tab.href
            ? 'bg-white/10 text-immich-dark-fg'
            : 'text-immich-dark-fg/50 hover:bg-white/5'}"
        >
          {tab.label}
        </a>
      {/each}
    </nav>
    {@render children()}
  </div>
</div>
