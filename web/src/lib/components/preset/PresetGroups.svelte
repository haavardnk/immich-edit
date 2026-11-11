<script lang="ts">
  import { CHIP, CHIP_ROW } from '#lib/components/editor/touch/chips.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { Tabs } from 'bits-ui';

  let { chips = false }: { chips?: boolean } = $props();
</script>

{#if chips}
  <Tabs.Root
    value={presetStrip.group ?? ''}
    onValueChange={(value) => presetStrip.pick(value === '' ? null : value.slice(1))}
    activationMode="manual"
  >
    <Tabs.List aria-label="Preset groups" class={CHIP_ROW}>
      {#each presetStrip.groupOptions as option (option.value)}
        <Tabs.Trigger value={option.value === null ? '' : `:${option.value}`} class={CHIP}
          >{option.label}</Tabs.Trigger
        >
      {/each}
    </Tabs.List>
  </Tabs.Root>
{:else}
  <nav
    aria-label="Preset groups"
    class="flex w-20 flex-none flex-col gap-px overflow-y-auto scrollbar-hidden"
    style:height="{ui.filmstripHeight}px"
  >
    {#each presetStrip.groupOptions as option (option.value)}
      <button
        type="button"
        class="flex-none truncate rounded-sm px-1.5 py-0.5 text-left text-[11px] outline-none focus-visible:ring-2 focus-visible:ring-primary {presetStrip.group ===
        option.value
          ? 'bg-light-950/10 text-dark'
          : 'text-dark/60 hover:text-dark'}"
        title={option.label}
        aria-pressed={presetStrip.group === option.value}
        onclick={() => presetStrip.pick(option.value)}>{option.label}</button
      >
    {/each}
  </nav>
{/if}
