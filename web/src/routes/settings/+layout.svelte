<script lang="ts">
  import { page } from '$app/state';
  import { Container } from '@immich/ui';

  let { children }: { children: import('svelte').Snippet } = $props();

  const diagnostics = $derived(page.url.pathname === '/settings/diagnostics');
  const title = $derived(diagnostics ? 'Diagnostics' : 'Settings');
</script>

<div class="flex h-full min-h-0 flex-col bg-editor-chrome">
  <header class="flex h-12 shrink-0 items-center border-b border-hairline bg-light px-3 sm:px-5">
    <h1 class="truncate text-sm font-semibold text-white">{title}</h1>
  </header>
  <div class="min-h-0 flex-1 overflow-y-auto">
    <Container center class="max-w-3xl px-4 py-6 sm:px-8">
      <div class="min-w-0">
        {@render children()}
      </div>
    </Container>
  </div>
</div>
