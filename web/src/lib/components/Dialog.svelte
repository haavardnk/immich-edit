<script lang="ts">
  import { IconButton, Modal, ModalBody, ModalFooter, ModalHeader } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { leaveBehind, sheetParts, type SheetParts } from '#lib/components/sheet/sheetGhost.ts';
  import { layout } from '#lib/device/layout.svelte.ts';

  let {
    title,
    subtitle,
    size = 'small',
    onClose,
    bodyClass = '',
    actions,
    footer,
    children
  }: {
    title: string;
    subtitle?: string;
    size?: 'tiny' | 'small' | 'medium' | 'large' | 'giant';
    onClose: () => void;
    bodyClass?: string;
    actions?: Snippet;
    footer?: Snippet;
    children: Snippet;
  } = $props();

  let sheet: SheetParts | null = null;
  let slidAway = false;

  const keepSheet: Attachment<HTMLElement> = (content) => {
    sheet = sheetParts(content);
    slidAway = false;
  };

  function closeSheet(next: boolean): void {
    if (next) return;
    slidAway = true;
    onClose();
  }

  $effect(() => () => {
    if (sheet && !slidAway && layout.mode === 'touch') leaveBehind(sheet);
  });
</script>

{#if layout.mode === 'touch'}
  <BottomSheet
    open
    onOpenChange={closeSheet}
    {@attach keepSheet}
    {title}
    {subtitle}
    close="icon"
    header={actions}
    {footer}
    bodyClass="px-5 py-5 {bodyClass}"
  >
    {@render children()}
  </BottomSheet>
{:else}
  <Modal {size} {onClose} closeOnBackdropClick>
    <ModalHeader class="border-b border-hairline bg-light-50 px-5 py-4">
      <div class="flex items-center gap-3">
        <div class="flex min-w-0 flex-none flex-col">
          <Dialog.Title class="text-base font-semibold text-light-950">
            {title}
          </Dialog.Title>
          {#if subtitle}
            <p class="text-xs text-dark/65">{subtitle}</p>
          {/if}
        </div>
        <div class="flex flex-1 items-center justify-end gap-1.5">
          {@render actions?.()}
        </div>
        <IconButton
          size="small"
          variant="ghost"
          color="secondary"
          icon={mdiClose}
          aria-label="close"
          class="flex-none"
          onclick={onClose}
        />
      </div>
    </ModalHeader>
    <ModalBody class="bg-light-50 py-5 {bodyClass}">
      {@render children()}
    </ModalBody>
    {#if footer}
      <ModalFooter class="border-t border-hairline bg-light-50 px-5 py-3">
        {@render footer()}
      </ModalFooter>
    {/if}
  </Modal>
{/if}
