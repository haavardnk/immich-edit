<script lang="ts">
  import SheetScrim from '#lib/components/sheet/SheetScrim.svelte';
  import { afterTransition } from '#lib/components/sheet/transitions.ts';
  import { holdModalSheet } from '#lib/stores/overlays.ts';
  import { Button } from '@immich/ui';
  import { Dialog } from 'bits-ui';
  import { tick, untrack, type Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    open,
    onOpenChange,
    title,
    caption = false,
    onExited,
    children,
    ...rest
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onExited?: () => void;
    title: string;
    caption?: boolean;
    children: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'children' | 'title'> = $props();

  let frame: HTMLElement | null = null;
  let content = $state<HTMLElement | null>(null);
  let leaving = $state(false);
  let closing = false;
  let opened = 0;

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
    const session = opened;
    const element = frame;
    if (element) {
      leaving = true;
      await tick();
      await afterTransition(element);
      if (session !== opened) return;
      if (frame !== element) {
        closing = false;
        leaving = false;
        return;
      }
    }
    onOpenChange(false);
    await tick();
    closing = false;
    leaving = false;
  }

  function dismiss(event: Event): void {
    event.preventDefault();
    void requestClose();
  }

  function autoFocus(event: Event): void {
    event.preventDefault();
    content?.focus({ preventScroll: true });
  }

  $effect.pre(() => {
    if (!open) return;
    untrack(() => {
      opened += 1;
      closing = false;
      leaving = false;
    });
  });

  $effect(() => {
    if (!open) return;
    return holdModalSheet();
  });
</script>

<Dialog.Root
  bind:open={() => open, () => undefined}
  onOpenChangeComplete={(next) => {
    if (!next) onExited?.();
  }}
>
  <Dialog.Portal>
    <SheetScrim {leaving} />
    <Dialog.Content
      bind:ref={content}
      interactOutsideBehavior="close"
      escapeKeydownBehavior="close"
      onEscapeKeydown={dismiss}
      onInteractOutside={dismiss}
      onOpenAutoFocus={autoFocus}
    >
      {#snippet child({ props })}
        {@const ending = 'data-ending-style' in props}
        <div
          {...rest}
          {...props}
          data-leaving={leaving || ending ? '' : undefined}
          style:pointer-events={ending ? 'none' : 'auto'}
          tabindex="-1"
          class="fixed right-[calc(var(--safe-right)+8px)] bottom-[calc(var(--safe-bottom)+8px)] left-[calc(var(--safe-left)+8px)] z-50 mx-auto flex max-h-[calc(var(--app-height)-var(--safe-top)-var(--safe-bottom)-16px)] max-w-lg flex-col gap-2 outline-none starting:[translate:0_150%] data-leaving:translate-y-[150%] motion-safe:transition-transform motion-safe:duration-slow motion-safe:ease-standard motion-safe:data-leaving:duration-base motion-safe:data-leaving:ease-exit"
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
