<script lang="ts">
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import { createAttachmentKey } from 'svelte/attachments';

  let {
    trigger,
    children,
    disabled = false,
    title
  }: {
    trigger: Snippet<[Record<string, unknown>]>;
    children: Snippet;
    disabled?: boolean;
    title?: string;
  } = $props();

  function nativeFocus(props: Record<string, unknown>): Record<string, unknown> {
    const { tabindex: _tabindex, ...rest } = props;
    return rest;
  }

  let open = $state(false);
  const sheet = presence(() => open);

  const touch = $derived(layout.mode === 'touch');
  const guard = clickGuard();

  function show(): void {
    if (disabled) return;
    guard.arm();
    open = true;
  }

  const touchTriggerProps = {
    [createAttachmentKey()]: gestures({
      recognizers: [longPressRecognizer({ enabled: () => !disabled, start: show })],
      touchAction: null
    }),
    [createAttachmentKey()]: guard.attachment,
    oncontextmenu: (event: MouseEvent) => {
      event.preventDefault();
      show();
    }
  };
</script>

{#if touch}
  {@render trigger(touchTriggerProps)}
  {#if sheet.mounted}
    <ActionSheet
      {open}
      onOpenChange={(next) => (open = next)}
      onExited={sheet.exited}
      title={title ?? 'Actions'}
      caption={title !== undefined}
    >
      {@render children()}
    </ActionSheet>
  {/if}
{:else}
  <Bits.Root>
    <Bits.Trigger {disabled}>
      {#snippet child({ props })}
        {@render trigger(nativeFocus(props))}
      {/snippet}
    </Bits.Trigger>
    <Bits.Portal>
      <Bits.Content
        collisionPadding={8}
        class="z-50 min-w-44 rounded-md border border-light-950/10 bg-light-100 py-1 text-dark shadow-popover dark:ring-1 dark:ring-black/20"
      >
        {@render children()}
      </Bits.Content>
    </Bits.Portal>
  </Bits.Root>
{/if}
