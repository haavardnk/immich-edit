<script lang="ts">
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
  import { tick, type Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    open,
    onOpenChange,
    title,
    titleHidden = false,
    modal = true,
    dismissable = true,
    dismissOnOutside,
    outsideExempt = null,
    detents = ['content'],
    detent = $bindable(0),
    detentLabels,
    close,
    header,
    footer,
    bodyClass = '',
    children,
    ...rest
  }: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: string;
    titleHidden?: boolean;
    modal?: boolean;
    dismissable?: boolean;
    dismissOnOutside?: boolean;
    outsideExempt?: HTMLElement | null;
    detents?: readonly Detent[];
    detent?: number;
    detentLabels?: readonly string[];
    close?: 'icon' | 'done';
    header?: Snippet;
    footer?: Snippet;
    bodyClass?: string;
    children: Snippet;
  } & Omit<HTMLAttributes<HTMLDivElement>, 'children' | 'title'> = $props();

  const uid = $props.id();
  const valueId = `${uid}-detent`;
  const guard = clickGuard();
  const inset = viewportInset();

  let content = $state<HTMLElement | null>(null);
  let available = $state(0);
  let topHeight = $state(0);
  let bodyHeight = $state(0);
  let bottomHeight = $state(0);
  let focusInside = $state(false);
  let closing = false;

  const outsideCloses = $derived(dismissOnOutside ?? (modal && dismissable));
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
    await motion.slideOut();
    onOpenChange(false);
    await tick();
    closing = false;
    if (open) await motion.settle(restHeight);
  }

  function escape(event: KeyboardEvent): void {
    event.preventDefault();
    void requestClose();
  }

  function outside(event: Event): void {
    event.preventDefault();
    if (event.target instanceof Node && outsideExempt?.contains(event.target)) return;
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
    if (!motion.dragging() && !closing) motion.rest(height);
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
      <SheetScrim />
    {/if}
    {#if open}
      <div
        class="pointer-events-none fixed inset-x-0 z-50 flex flex-col pt-[calc(var(--safe-top)+8px)]"
        style:top="{inset.top}px"
        style:bottom="{inset.bottom}px"
      >
        <div class="flex min-h-0 flex-1 flex-col justify-end" bind:clientHeight={available}>
          <Dialog.Content
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
                {@attach trackFocus}
              >
                <div
                  class="flex min-h-0 flex-col overflow-hidden rounded-t-2xl bg-light-50 pr-(--safe-right) pl-(--safe-left) text-dark shadow-popover starting:translate-y-full motion-safe:transition-[translate,transform] motion-safe:duration-300 motion-safe:ease-out motion-safe:data-snapping:transition-[translate,transform,height]"
                  {@attach motion.panel}
                >
                  <div
                    class="flex-none select-none"
                    bind:offsetHeight={topHeight}
                    {@attach guard.attachment}
                  >
                    <div class="flex justify-center pt-1" {@attach handleDrag}>
                      <button
                        type="button"
                        data-sheet-handle
                        aria-label="Resize sheet"
                        aria-describedby={steppable ? valueId : undefined}
                        aria-hidden={steppable ? undefined : 'true'}
                        tabindex={steppable ? 0 : -1}
                        class="relative flex h-5 w-24 cursor-grab items-center justify-center rounded-full outline-none before:absolute before:inset-x-0 before:-inset-y-3 focus-visible:ring-2 focus-visible:ring-primary"
                        onclick={step}
                      >
                        <span class="h-1 w-9 rounded-full bg-light-950/25"></span>
                      </button>
                      {#if steppable}
                        <span id={valueId} class="sr-only">{detentText}</span>
                      {/if}
                    </div>
                    {#if headerRow}
                      <div
                        class="flex min-h-12 items-center gap-1 border-b border-hairline pr-1.5 pl-4"
                      >
                        <div class="flex min-h-11 min-w-0 shrink items-center" {@attach titleDrag}>
                          <Dialog.Title
                            class={titleHidden
                              ? 'sr-only'
                              : 'min-w-0 truncate text-base font-semibold text-light-950'}
                          >
                            {title}
                          </Dialog.Title>
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
                            aria-label="close"
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
                      <div class="flex items-center border-t border-hairline px-5 py-3">
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
