<script lang="ts">
  import SheetHandle from '#lib/components/sheet/SheetHandle.svelte';
  import SheetScrim from '#lib/components/sheet/SheetScrim.svelte';
  import { sheetMotion } from '#lib/components/sheet/sheetMotion.ts';
  import { nextDetent, resolveDetents, type Detent } from '#lib/components/sheet/sheetSnap.ts';
  import { viewportInset, windowViewport } from '#lib/components/sheet/visualViewport.svelte.ts';
  import { clickGuard } from '#lib/gestures/clickGuard.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { holdModalSheet } from '#lib/stores/overlays.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import { tick, untrack, type Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    open,
    onOpenChange,
    title,
    subtitle,
    titleHidden = false,
    modal = true,
    scrim = 'dim',
    dismissable = true,
    detents = ['content'],
    detent = $bindable(0),
    detentLabels,
    close,
    closeLabel = 'close',
    header,
    footer,
    bodyClass = '',
    onExited,
    children,
    ...rest
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onExited?: () => void;
    title: string;
    subtitle?: string;
    titleHidden?: boolean;
    modal?: boolean;
    scrim?: 'dim' | 'light';
    dismissable?: boolean;
    detents?: readonly Detent[];
    detent?: number;
    detentLabels?: readonly string[];
    close?: 'icon' | 'done';
    closeLabel?: string;
    header?: Snippet;
    footer?: Snippet;
    bodyClass?: string;
    children: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'children' | 'title'> = $props();

  const guard = clickGuard();
  const inset = viewportInset();

  let content = $state<HTMLElement | null>(null);
  let available = $state(0);
  let topHeight = $state(0);
  let bodyHeight = $state(0);
  let bottomHeight = $state(0);
  let focusInside = $state(false);
  let present = $state(untrack(() => open));
  let leaving = $state(false);
  let closing = false;
  let slidOut = false;
  let exits = 0;
  let opened = 0;

  const outsideCloses = $derived(modal && dismissable);
  const escapeCloses = $derived(dismissable && (modal || focusInside));
  const headerRow = $derived(!titleHidden || header !== undefined || close !== undefined);
  const steppable = $derived(detents.length > 1);
  const current = $derived(Math.min(Math.max(0, detent), Math.max(0, detents.length - 1)));
  const heights = $derived(
    resolveDetents(detents, { available, content: topHeight + bodyHeight + bottomHeight })
  );
  const restHeight = $derived(available > 0 ? (heights[current] ?? 0) : 0);
  const detentText = $derived(
    detentLabels?.[current] ?? `Size ${current + 1} of ${detents.length}`
  );

  const motion = sheetMotion({
    heights: () => heights,
    rest: () => restHeight,
    available: () => available,
    dismissable: () => dismissable,
    grab: guard.arm,
    snap: (index) => (detent = index),
    dismiss: () => void requestClose()
  });

  function dragArea(): Attachment<HTMLElement> {
    return gestures({
      recognizers: [
        panRecognizer({
          lockAxis: true,
          start: motion.begin,
          move: motion.follow,
          end: motion.release,
          cancel: motion.abort
        })
      ]
    });
  }

  const handleDrag = dragArea();
  const titleDrag = dragArea();

  const trackFocus: Attachment<HTMLElement> = (element) => {
    const listening = new AbortController();
    const options = { signal: listening.signal };
    element.addEventListener('focusin', () => (focusInside = true), options);
    element.addEventListener(
      'focusout',
      (event) => {
        focusInside = event.relatedTarget instanceof Node && element.contains(event.relatedTarget);
      },
      options
    );
    return () => {
      listening.abort();
      focusInside = false;
    };
  };

  async function requestClose(): Promise<void> {
    if (closing || !open) return;
    closing = true;
    leaving = true;
    const session = opened;
    await motion.slideOut();
    if (session !== opened) return;
    slidOut = true;
    onOpenChange(false);
    await tick();
    closing = false;
    if (open) reopen();
  }

  function reopen(): void {
    opened += 1;
    exits += 1;
    closing = false;
    slidOut = false;
    present = true;
    if (!leaving) return;
    leaving = false;
    void motion.settle(restHeight);
  }

  async function exit(): Promise<void> {
    const mine = ++exits;
    leaving = true;
    if (!slidOut) await motion.slideOut();
    if (mine !== exits || open) return;
    present = false;
    leaving = false;
    slidOut = false;
    onExited?.();
  }

  function escape(event: KeyboardEvent): void {
    event.preventDefault();
    void requestClose();
  }

  function outside(event: Event): void {
    event.preventDefault();
    void requestClose();
  }

  function step(): void {
    if (!steppable) return;
    const next = nextDetent(current, detents.length);
    detent = next;
    void motion.settle(heights[next] ?? 0);
  }

  function autoFocus(event: Event): void {
    event.preventDefault();
    if (!modal || !content || content.contains(document.activeElement)) return;
    content.focus({ preventScroll: true });
  }

  $effect(() => {
    const height = restHeight;
    if (!motion.dragging() && !closing && !untrack(() => leaving)) motion.rest(height);
  });

  $effect(() => {
    if (open) untrack(reopen);
    else if (untrack(() => present)) untrack(() => void exit());
  });

  $effect(() => {
    if (!open || !modal) return;
    return holdModalSheet();
  });

  $effect(() => {
    if (!open) return;
    return inset.watch(windowViewport());
  });
</script>

<Dialog.Root bind:open={() => open, () => undefined}>
  <Dialog.Portal>
    {#if modal}
      <SheetScrim tone={scrim} {leaving} />
    {/if}
    {#if present}
      <div
        class="pointer-events-none fixed inset-x-0 z-50 flex flex-col pt-[calc(var(--safe-top)+var(--sheet-gap))]"
        style:top="{inset.top}px"
        style:bottom="{inset.bottom}px"
      >
        <div class="flex min-h-0 flex-1 flex-col justify-end" bind:clientHeight={available}>
          <Dialog.Content
            forceMount
            bind:ref={content}
            trapFocus={modal}
            preventScroll={modal}
            interactOutsideBehavior={outsideCloses ? 'close' : 'ignore'}
            escapeKeydownBehavior={escapeCloses ? 'close' : 'ignore'}
            onEscapeKeydown={escape}
            onInteractOutside={outside}
            onOpenAutoFocus={autoFocus}
          >
            {#snippet child({ props })}
              <div
                {...rest}
                {...props}
                aria-modal={modal ? 'true' : 'false'}
                tabindex="-1"
                class="flex max-h-full min-h-0 w-full flex-col outline-none"
                style:pointer-events={leaving && !open ? 'none' : 'auto'}
                {@attach trackFocus}
              >
                <div
                  class="flex min-h-0 flex-col overflow-hidden rounded-t-2xl bg-light-50 pr-(--safe-right) pl-(--safe-left) text-dark shadow-popover starting:[translate:0_100%] motion-safe:transition-[translate,transform] motion-safe:duration-slow motion-safe:ease-standard motion-safe:data-snapping:transition-[translate,transform,height] motion-safe:data-leaving:duration-base motion-safe:data-leaving:ease-exit"
                  {@attach motion.panel}
                >
                  <div
                    class="flex-none select-none"
                    bind:offsetHeight={topHeight}
                    {@attach guard.attachment}
                  >
                    <SheetHandle {steppable} {detentText} drag={handleDrag} onstep={step} />
                    {#if headerRow}
                      <div
                        class="flex min-h-12 items-center gap-1 border-b border-hairline pr-1.5 pl-4"
                      >
                        <div
                          class="flex min-h-11 min-w-0 shrink flex-wrap items-center gap-x-2"
                          {@attach titleDrag}
                        >
                          <Dialog.Title
                            class={titleHidden
                              ? 'sr-only'
                              : 'min-w-0 truncate text-base font-semibold text-light-950'}
                          >
                            {title}
                          </Dialog.Title>
                          {#if subtitle}
                            <span class="shrink-0 text-[13px] text-dark/65">{subtitle}</span>
                          {/if}
                        </div>
                        <div class="flex min-w-0 flex-1 items-center justify-end gap-1.5">
                          {@render header?.()}
                        </div>
                        {#if close === 'icon'}
                          <IconButton
                            size="small"
                            variant="ghost"
                            color="secondary"
                            icon={mdiClose}
                            aria-label={closeLabel}
                            class="min-h-11 min-w-11 flex-none"
                            onclick={() => void requestClose()}
                          />
                        {:else if close === 'done'}
                          <Button
                            size="small"
                            variant="ghost"
                            color="primary"
                            class="min-h-11 min-w-11 flex-none font-semibold"
                            onclick={() => void requestClose()}
                          >
                            Done
                          </Button>
                        {/if}
                      </div>
                    {:else}
                      <Dialog.Title class="sr-only">{title}</Dialog.Title>
                    {/if}
                  </div>
                  <div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
                    <div class={bodyClass} bind:offsetHeight={bodyHeight}>
                      {@render children()}
                    </div>
                  </div>
                  <div
                    class={inset.bottom > 0 ? 'flex-none' : 'flex-none pb-(--safe-bottom)'}
                    bind:offsetHeight={bottomHeight}
                  >
                    {#if footer}
                      <div class="flex items-center border-t border-hairline px-3 py-2.5">
                        {@render footer()}
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            {/snippet}
          </Dialog.Content>
        </div>
      </div>
    {/if}
  </Dialog.Portal>
</Dialog.Root>
