<script lang="ts">
  import { page } from '$app/state';
  import SearchField from '#lib/components/SearchField.svelte';
  import SettingsSearchResults from '#lib/components/settings/SettingsSearchResults.svelte';
  import { settingsSearchHost } from '#lib/commands/settings.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { install } from '#lib/pwa/install.svelte.ts';
  import { visiblePages } from '#lib/settings/pages.ts';
  import { revealAnchor } from '#lib/settings/revealAnchor.ts';
  import { searchSettings, type SettingsHit } from '#lib/settings/searchSettings.ts';
  import { SETTINGS_INDEX } from '#lib/settings/settingsIndex.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { Popover } from 'bits-ui';
  import { onMount, type Snippet } from 'svelte';

  let { children }: { children?: Snippet } = $props();

  let query = $state('');
  let open = $state(false);
  let field = $state<HTMLElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);
  let results = $state<HTMLElement | null>(null);

  const inline = $derived(layout.compact);
  const pages = $derived(visiblePages(session.isAdmin));
  const searching = $derived(query.trim() !== '');
  const hits = $derived(
    searchSettings(query, SETTINGS_INDEX, {
      pages,
      desktop: layout.mode === 'desktop',
      installable: install.state.kind !== 'hidden'
    })
  );

  onMount(() =>
    settingsSearchHost.attach({
      focus: () => {
        input?.focus();
        open = searching;
      }
    })
  );

  function hitLinks(): HTMLElement[] {
    return [...(results?.querySelectorAll<HTMLElement>('[data-settings-hit]') ?? [])];
  }

  function typed(next: string): void {
    open = next.trim() !== '';
  }

  function fieldKeys(event: KeyboardEvent): void {
    if (event.key !== 'ArrowDown' && !(event.key === 'Enter' && searching)) return;
    open = searching;
    const [first] = hitLinks();
    if (!first) return;
    event.preventDefault();
    if (event.key === 'Enter') first.click();
    else first.focus();
  }

  function resultKeys(event: KeyboardEvent): void {
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    const links = hitLinks();
    const index = links.indexOf(event.target as HTMLElement);
    if (index < 0) return;
    event.preventDefault();
    const next = index + (event.key === 'ArrowDown' ? 1 : -1);
    if (next < 0) input?.focus();
    else links[Math.min(next, links.length - 1)]?.focus();
  }

  function picked(hit: SettingsHit): void {
    open = false;
    const anchor = hit.entry.anchor;
    if (anchor && hit.href === `${page.url.pathname}${page.url.hash}`) {
      revealAnchor(document.body, anchor);
    }
  }
</script>

{#snippet list()}
  <div bind:this={results} data-sveltekit-replacestate={inline ? undefined : ''}>
    {#if hits.length === 0}
      <p class="px-3 py-2 text-xs text-dark/80 touch:px-4 touch:text-sm">
        No settings match “{query.trim()}”.
      </p>
    {:else}
      <SettingsSearchResults {hits} {pages} {query} onpick={picked} onkeydown={resultKeys} />
    {/if}
  </div>
{/snippet}

<div bind:this={field}>
  <SearchField
    bind:value={query}
    bind:ref={input}
    label="Search settings"
    oninput={typed}
    onkeydown={fieldKeys}
  />
</div>

{#if inline}
  {#if searching}
    <div class="overflow-hidden rounded-2xl bg-light-100">{@render list()}</div>
  {:else}
    {@render children?.()}
  {/if}
{:else}
  {@render children?.()}
  <Popover.Root bind:open={() => open && searching, (next) => (open = next)}>
    <Popover.Portal>
      <Popover.Content
        customAnchor={field}
        side="bottom"
        align="start"
        sideOffset={6}
        trapFocus={false}
        onOpenAutoFocus={(event) => event.preventDefault()}
        onCloseAutoFocus={(event) => {
          event.preventDefault();
          if (results?.contains(document.activeElement)) input?.focus();
        }}
        onInteractOutside={(event) => {
          if (field?.contains(event.target as Node)) event.preventDefault();
        }}
        class="z-50 max-h-(--bits-popover-content-available-height) w-[26rem] overflow-y-auto rounded-xl border border-hairline bg-light-100 shadow-popover"
      >
        {@render list()}
        {#if layout.mode === 'desktop'}
          <p
            class="border-t border-hairline px-3 py-1.5 text-[10.5px] text-dark/80"
            aria-hidden="true"
          >
            ↑↓ move · ↵ open · esc close · {keysFor('settingsSearch')} search
          </p>
        {/if}
      </Popover.Content>
    </Popover.Portal>
  </Popover.Root>
{/if}
