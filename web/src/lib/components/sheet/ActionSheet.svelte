<script lang="ts">
  import SheetScrim from '#lib/components/sheet/SheetScrim.svelte';
  import { afterTransition } from '#lib/components/sheet/transitions.ts';
  import { holdModalSheet } from '#lib/stores/overlays.ts';
  import { Button } from '@immich/ui';
  import { Dialog } from 'bits-ui';
  import { tick, type Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    open,
    onOpenChange,
    title,
    caption = false,
    children,
    ...rest
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: string;
    caption?: boolean;
    children: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'children' | 'title'> = $props();

  let frame: HTMLElement | null = null;
  let content = $state<HTMLElement | null>(null);
  let closing = false;

  const frameRef: Attachment<HTMLElement> = (element) => {
    frame = element;
    return () => {
      if (frame === element) frame = null;
    };
  };

  const closeAfterRow: Attachment<HTMLElement> = (element) => {
    const listening = new AbortController();
    element.addEventListener(
      'click',
      (event) => {
        const row =
          event.target instanceof Element ? event.target.closest('[data-menu-row]') : null;
        if (!row || row.hasAttribute('data-menu-stay')) return;
        void requestClose();
      },
      { signal: listening.signal }
    );
    return () => listening.abort();
  };

  async function requestClose(): Promise<void> {
    if (closing || !open) return;
    closing = true;
    const element = frame;
    if (element) {
      element.dataset.leaving = '';
      await afterTransition(element);
    }
    onOpenChange(false);
    await tick();
    closing = false;
    if (element) delete element.dataset.leaving;
  }

  function dismiss(event: Event): void {
    event.preventDefault();
    void requestClose();
  }

  function autoFocus(event: Event): void {
    event.preventDefault();
    content?.focus({ preventScroll: true });
  }

  $effect(() => {
    if (!open) return;
    return holdModalSheet();
  });
</script>

<Dialog.Root bind:open={() => open, () => undefined}>
  <Dialog.Portal>
    <SheetScrim />
    <Dialog.Content
      bind:ref={content}
      interactOutsideBehavior="close"
      escapeKeydownBehavior="close"
      onEscapeKeydown={dismiss}
      onInteractOutside={dismiss}
      onOpenAutoFocus={autoFocus}
    >
      {#snippet child({ props })}
        <div
          {...rest}
          {...props}
          tabindex="-1"
          class="fixed right-[calc(var(--safe-right)+8px)] bottom-[calc(var(--safe-bottom)+8px)] left-[calc(var(--safe-left)+8px)] z-50 mx-auto flex max-h-[calc(100dvh-var(--safe-top)-var(--safe-bottom)-16px)] max-w-lg flex-col gap-2 outline-none starting:translate-y-[150%] data-leaving:translate-y-[150%] motion-safe:transition-transform motion-safe:duration-300 motion-safe:ease-out"
          {@attach frameRef}
        >
          <div class="flex min-h-0 flex-col overflow-hidden rounded-[14px] bg-light-100 text-dark">
            <Dialog.Title
              class={caption
                ? 'flex-none border-b border-hairline px-4 pt-3 pb-2 text-center text-[12.5px] font-normal text-dark/65'
                : 'sr-only'}
            >
              {title}
            </Dialog.Title>
            <div
              class="flex min-h-0 flex-col overflow-y-auto overscroll-contain"
              {@attach closeAfterRow}
            >
              {@render children()}
            </div>
          </div>
          <Button
            size="large"
            variant="ghost"
            color="primary"
            fullWidth
            class="h-13 flex-none rounded-[14px] bg-light-100 text-base font-semibold"
            onclick={() => void requestClose()}
          >
            Cancel
          </Button>
        </div>
      {/snippet}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
