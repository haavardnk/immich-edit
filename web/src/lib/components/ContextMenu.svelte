<script lang="ts">
  import { ContextMenu as Bits } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import { createAttachmentKey } from 'svelte/attachments';

  let {
    trigger,
    children,
    disabled = false
  }: {
    trigger: Snippet<[Record<string, unknown>]>;
    children: Snippet;
    disabled?: boolean;
  } = $props();

  let open = $state(false);

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
  {#if open}
    <ActionSheet {open} onOpenChange={(next) => (open = next)} title="Actions">
      {@render children()}
    </ActionSheet>
  {/if}
{:else}
  <Bits.Root>
    <Bits.Trigger {disabled}>
      {#snippet child({ props })}
        {@render trigger(props)}
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
