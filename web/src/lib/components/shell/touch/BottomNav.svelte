<script lang="ts">
  import { page } from '$app/state';
  import { activeTab, NAV_TABS } from '#lib/components/shell/touch/navTabs.ts';
  import { touchBars } from '#lib/components/shell/touch/touchBars.svelte.ts';
  import { Icon } from '@immich/ui';

  const current = $derived(activeTab(page.url.pathname));
</script>

<nav
  aria-label="Main"
  inert={touchBars.hidden}
  data-tab-bar
  class="flex shrink-0 border-t border-hairline bg-light-50 pb-(--safe-bottom) motion-safe:transition-[margin] motion-safe:duration-200 {touchBars.hidden
    ? '-mb-[calc(3.5rem+1px+var(--safe-bottom))]'
    : ''}"
>
  {#each NAV_TABS as tab (tab.id)}
    {@const active = current === tab.id}
    <a
      href={tab.href}
      aria-current={active ? 'page' : undefined}
      class="flex h-14 min-w-0 flex-1 flex-col items-center justify-center gap-0.5 text-[11px] font-medium outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {active
        ? 'text-primary'
        : 'text-dark/65'}"
    >
      <span
        class="flex h-7 w-14 items-center justify-center rounded-full {active
          ? 'bg-primary/15'
          : ''}"
      >
        <Icon icon={tab.icon} size="22px" aria-hidden="true" />
      </span>
      {tab.label}
    </a>
  {/each}
</nav>
