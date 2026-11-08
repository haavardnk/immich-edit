<script lang="ts">
  import { Tooltip } from '@immich/ui';
  import { Popover as Bits, mergeProps } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { providePickerHost } from '#lib/components/sheet/pickerHost.ts';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { holdEscape } from '#lib/stores/overlays.ts';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';
  type Appearance = 'menu' | 'panel' | 'control';

  let {
    open,
    anchor = 'bottom',
    align = 'start',
    onOpenChange,
    trigger,
    children,
    appearance = 'menu',
    contentClass = '',
    sheetClass = '',
    sheetPicker = false,
    title,
    subtitle,
    tooltip,
    customAnchor
  }: {
    open: boolean;
    anchor?: Anchor;
    align?: Align;
    onOpenChange: (open: boolean) => void;
    trigger?: Snippet<[Record<string, unknown>]>;
    children: Snippet;
    appearance?: Appearance;
    contentClass?: string;
    sheetClass?: string;
    sheetPicker?: boolean;
    title?: string;
    subtitle?: string;
    tooltip?: string;
    customAnchor?: HTMLElement | null;
  } = $props();

  const appearanceClass = $derived(
    appearance === 'control'
      ? 'flex h-10 items-center gap-1 overflow-visible rounded-lg border border-hairline bg-light-100 px-1 text-dark shadow-popover'
      : 'max-h-(--bits-popover-content-available-height) overflow-y-auto rounded-md border border-light-950/10 bg-light-100 text-dark shadow-popover dark:ring-1 dark:ring-black/20'
  );

  let tooltipOpen = $state(false);
  const sheet = presence(() => open);

  const sheetTriggerProps = $derived({
    'aria-haspopup': 'dialog',
    'aria-expanded': open,
    onclick: () => onOpenChange(!open)
  });

  providePickerHost({
    get inline() {
      return sheetPicker;
    }
  });

  $effect(() => {
    if (open) tooltipOpen = false;
  });

  $effect(() => {
    if (open && layout.mode !== 'touch') return holdEscape();
  });
</script>

{#snippet tooltipTrigger(props: Record<string, unknown>)}
  <Tooltip
    text={tooltip}
    bind:open={() => tooltipOpen && !open, (next) => (tooltipOpen = next && !open)}
  >
    {#snippet child({ props: tooltipProps })}
      {@render trigger?.(mergeProps(tooltipProps, props))}
    {/snippet}
  </Tooltip>
{/snippet}

{#if layout.mode === 'touch'}
  {@render tooltipTrigger(sheetTriggerProps)}
  {#if sheet.mounted && appearance === 'menu'}
    <ActionSheet
      {open}
      {onOpenChange}
      onExited={sheet.exited}
      title={title ?? 'Menu'}
      caption={title !== undefined}
      data-popover-content
    >
      {@render children()}
    </ActionSheet>
  {:else if sheet.mounted && appearance === 'panel'}
    <BottomSheet
      {open}
      {onOpenChange}
      onExited={sheet.exited}
      title={title ?? 'Options'}
      {subtitle}
      titleHidden={title === undefined}
      close="done"
      detents={sheetPicker ? [{ fraction: 1 }] : undefined}
      bodyClass={sheetClass || 'px-4 py-3'}
      data-popover-content
    >
      {@render children()}
    </BottomSheet>
  {:else if sheet.mounted}
    <BottomSheet
      {open}
      {onOpenChange}
      onExited={sheet.exited}
      title={title ?? 'Options'}
      titleHidden={title === undefined}
      close="icon"
      closeLabel="Close"
      scrim="light"
      bodyClass="flex flex-wrap items-center justify-center gap-1 px-4 py-3 {sheetClass}"
      data-popover-content
    >
      {@render children()}
    </BottomSheet>
  {/if}
{:else}
  <Bits.Root bind:open={() => open, onOpenChange}>
    {#if trigger}
      <Bits.Trigger>
        {#snippet child({ props })}
          {@render tooltipTrigger(props)}
        {/snippet}
      </Bits.Trigger>
    {/if}
    <Bits.Portal>
      <Bits.Content
        data-popover-content
        side={anchor}
        {customAnchor}
        {align}
        sideOffset={4}
        collisionPadding={8}
        class="z-50 {appearanceClass} {contentClass}"
      >
        {@render children()}
      </Bits.Content>
    </Bits.Portal>
  </Bits.Root>
{/if}
