<script lang="ts">
  import { page } from '$app/state';
  import { mdiCogOutline, mdiStethoscope } from '@mdi/js';
  import SidebarLink from './SidebarLink.svelte';

  const currentPath = $derived(page.url.pathname);
</script>

<aside
  aria-label="Administration"
  class="flex h-full w-64 min-h-0 flex-col overflow-hidden bg-neutral-950 text-dark"
>
  <div class="shrink-0 px-5 pb-2 pt-5 text-[10px] font-semibold uppercase text-white/35">
    Administration
  </div>
  <nav aria-label="Settings" class="flex-1 min-h-0 space-y-1 overflow-y-auto pe-5">
    <SidebarLink
      href="/settings"
      icon={mdiCogOutline}
      label="Settings"
      active={currentPath === '/settings'}
    />
    <div class="mx-4 my-3 h-px bg-hairline"></div>
    <SidebarLink
      href="/settings/diagnostics"
      icon={mdiStethoscope}
      label="Diagnostics"
      active={currentPath === '/settings/diagnostics'}
    />
  </nav>
</aside>
