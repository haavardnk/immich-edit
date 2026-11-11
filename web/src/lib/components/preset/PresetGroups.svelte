<script lang="ts">
  import SearchField from '#lib/components/SearchField.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { CHIP, CHIP_ROW } from '#lib/components/editor/touch/chips.ts';
  import { FAVOURITE_PRESETS, presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiStar } from '@mdi/js';
  import { Tabs } from 'bits-ui';

  let { chips = false }: { chips?: boolean } = $props();
</script>

{#if chips}
  <Tabs.Root value={presetStrip.group} onValueChange={presetStrip.pick} activationMode="manual">
    <Tabs.List aria-label="Preset groups" class={CHIP_ROW}>
      {#each presetStrip.groupOptions as option (option.key)}
        <Tabs.Trigger value={option.key} class={CHIP}>{option.label}</Tabs.Trigger>
      {/each}
    </Tabs.List>
  </Tabs.Root>
{:else}
  <div class="flex w-[150px] flex-none flex-col gap-1" style:height="{ui.filmstripHeight}px">
    <SearchField bind:value={presetStrip.query} label="Search presets" compact />
    {#if presetStrip.filterable}
      <nav
        aria-label="Preset groups"
        class="flex min-h-0 flex-1 flex-col gap-px overflow-y-auto scrollbar-hidden"
      >
        {#each presetStrip.groupOptions as option (option.key)}
          {@const pressed = !presetStrip.searching && presetStrip.group === option.key}
          <TruncationTooltip text={option.label}>
            {#snippet child({ props })}
              <button
                {...props}
                type="button"
                class="flex flex-none items-center gap-1 rounded-sm px-1.5 py-0.5 text-left text-[11px] outline-none focus-visible:ring-2 focus-visible:ring-primary {pressed
                  ? 'bg-light-950/10 text-dark'
                  : 'text-dark/60 hover:text-dark'}"
                aria-pressed={pressed}
                onclick={() => presetStrip.pick(option.key)}
              >
                {#if option.key === FAVOURITE_PRESETS}
                  <Icon icon={mdiStar} size="12px" class="flex-none" aria-hidden />
                {/if}
                <span data-truncate class="truncate">{option.label}</span>
              </button>
            {/snippet}
          </TruncationTooltip>
        {/each}
      </nav>
    {/if}
  </div>
{/if}
