<script lang="ts">
  import { page } from '$app/state';
  import { activeTab, NAV_TABS } from '#lib/components/shell/touch/navTabs.ts';
  import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
  import { TAB_BAR_BOTTOM } from '#lib/components/shell/touch/touchChrome.ts';
  import { Icon } from '@immich/ui';

  const current = $derived(activeTab(page.url.pathname));
</script>

<nav
  aria-label="Main"
  inert={touchBars.hidden}
  data-tab-bar
  class="flex shrink-0 justify-around border-t border-hairline bg-light-50 pb-(--safe-bottom) motion-safe:transition-[margin] motion-safe:duration-200"
  style:margin-bottom={touchBars.hidden ? `calc(-1 * ${TAB_BAR_BOTTOM})` : null}
>
  {#each NAV_TABS as tab (tab.id)}
    {@const active = current === tab.id}
    <a
      href={tab.href}
      aria-current={active ? 'page' : undefined}
      class="group flex h-15 min-w-0 flex-1 items-center justify-center outline-none"
    >
      <span
        class="flex h-13 w-20 max-w-full flex-col items-center justify-center gap-0.5 rounded-xl text-[11px] font-medium group-focus-visible:ring-2 group-focus-visible:ring-primary {active
          ? 'bg-primary/12 text-light-950'
          : 'text-dark/65'}"
      >
        <Icon icon={tab.icon} size="22px" class={active ? 'text-primary' : ''} aria-hidden="true" />
        {tab.label}
      </span>
    </a>
  {/each}
</nav>
