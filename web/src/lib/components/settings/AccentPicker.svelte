<script lang="ts">
  import { appearance } from '#lib/stores/appearance.svelte.ts';
  import { themeEntry } from '#lib/theme/registry.ts';
  import { resolveTheme } from '#lib/theme/resolve.ts';
  import type { Scheme } from '#lib/theme/types.ts';
  import { RadioGroup } from 'bits-ui';

  let { scheme, label }: { scheme: Scheme; label: string } = $props();

  const entry = $derived(
    themeEntry(
      scheme === 'light' ? appearance.current.lightTheme : appearance.current.darkTheme,
      scheme
    )
  );

  const selected = $derived(
    resolveTheme(entry.family, entry.theme, appearance.current.accents[scheme]).accent
  );

  const swatches = $derived(
    entry.family.accents.map((accent) => ({
      ...accent,
      color: resolveTheme(entry.family, entry.theme, accent.id).accentColor
    }))
  );
</script>

{#if swatches.length > 1}
  <section class="space-y-2">
    <h3 class="text-xs font-medium">{label}</h3>
    <RadioGroup.Root
      value={selected}
      onValueChange={(accent) => appearance.setAccent(scheme, accent)}
      orientation="horizontal"
      aria-label={label}
      class="flex flex-wrap gap-2"
    >
      {#each swatches as accent (accent.id)}
        <RadioGroup.Item
          value={accent.id}
          aria-label={accent.name}
          title={accent.name}
          class="size-7 rounded-full ring-1 ring-hairline ring-offset-2 ring-offset-light outline-none transition-shadow hover:ring-light-950/35 focus-visible:ring-2 focus-visible:ring-primary aria-checked:ring-2 aria-checked:ring-dark"
          style="background-color: {accent.color}"
        />
      {/each}
    </RadioGroup.Root>
  </section>
{/if}
