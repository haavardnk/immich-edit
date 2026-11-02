<script lang="ts">
  import { RadioGroup } from 'bits-ui';
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { themeEntry, themesFor } from '#lib/theme/registry.ts';
  import { resolveTheme } from '#lib/theme/resolve.ts';
  import type { Scheme } from '#lib/theme/types.ts';

  let { scheme, label }: { scheme: Scheme; label: string } = $props();

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
  <h2 class="text-xs font-medium">{label}</h2>
  <RadioGroup.Root
    value={selected}
    onValueChange={(id) => appearance.setTheme(scheme, id)}
    aria-label={label}
    class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4"
  >
    {#each tiles as tile (tile.id)}
      <RadioGroup.Item
        value={tile.id}
        aria-label={tile.name}
        class="group flex min-w-0 flex-col gap-1.5 rounded-lg text-left outline-none"
      >
        <span
          aria-hidden="true"
          class="relative block h-16 w-full overflow-hidden rounded-lg ring-1 ring-hairline transition-shadow group-hover:ring-light-950/35 group-focus-visible:ring-2 group-focus-visible:ring-primary group-aria-checked:ring-2 group-aria-checked:ring-primary"
          style:background-color={tile.palette.base}
        >
          <span class="absolute inset-x-0 top-0 h-3" style:background-color={tile.palette.mantle}
          ></span>
          <span
            class="absolute top-6 left-2.5 h-1.5 w-14 rounded-full"
            style:background-color={tile.palette.text}
          ></span>
          <span
            class="absolute top-9.5 left-2.5 h-1.5 w-9 rounded-full"
            style:background-color={tile.palette.subtext}
          ></span>
          <span
            class="absolute right-2.5 bottom-2.5 size-4 rounded-full"
            style:background-color={tile.accent}
          ></span>
        </span>
        <span class="truncate text-xs text-dark/80 group-aria-checked:text-primary">
          {tile.name}
        </span>
      </RadioGroup.Item>
    {/each}
  </RadioGroup.Root>
</section>
