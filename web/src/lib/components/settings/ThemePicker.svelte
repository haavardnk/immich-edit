<script lang="ts">
  import { RadioGroup } from 'bits-ui';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { themeEntry, themesFor } from '#lib/theme/registry.ts';
  import { resolveTheme } from '#lib/theme/resolve.ts';
  import type { Scheme } from '#lib/theme/types.ts';
  import ThemeSwatch from '#lib/components/settings/ThemeSwatch.svelte';

  let { scheme, label, bare = false }: { scheme: Scheme; label: string; bare?: boolean } = $props();

  const selected = $derived(
    themeEntry(
      scheme === 'light' ? appearance.current.lightTheme : appearance.current.darkTheme,
      scheme
    ).theme.id
  );

  const tiles = $derived(
    themesFor(scheme).map(({ family, theme }) => {
      const resolved = resolveTheme(family, theme, appearance.current.accents[scheme]);
      return {
        id: theme.id,
        name: theme.name,
        palette: resolved.palette,
        accent: resolved.accentColor
      };
    })
  );
</script>

<section class="space-y-2">
  {#if !bare}
    <h2 class="text-xs font-medium">{label}</h2>
  {/if}
  <RadioGroup.Root
    value={selected}
    onValueChange={(id) => appearance.setTheme(scheme, id)}
    aria-label={label}
    class="grid grid-cols-3 gap-2 sm:gap-3 lg:grid-cols-4"
  >
    {#each tiles as tile (tile.id)}
      <RadioGroup.Item
        value={tile.id}
        aria-label={tile.name}
        class="group flex min-w-0 flex-col gap-1.5 rounded-lg text-left outline-none"
      >
        <ThemeSwatch palette={tile.palette} accent={tile.accent} />
        <span class="truncate text-xs text-dark/80 group-aria-checked:text-primary">
          {tile.name}
        </span>
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>
</section>
