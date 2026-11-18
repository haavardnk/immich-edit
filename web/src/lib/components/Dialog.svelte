<script lang="ts">
  import { IconButton, Modal, ModalBody, ModalFooter, ModalHeader } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';

  let {
    title,
    size = 'small',
    onClose,
    bodyClass = '',
    actions,
    footer,
    children
  }: {
    title: string;
    size?: 'tiny' | 'small' | 'medium' | 'large' | 'giant';
    onClose: () => void;
    bodyClass?: string;
    actions?: Snippet;
    footer?: Snippet;
    children: Snippet;
  } = $props();

  function closeSheet(next: boolean): void {
    if (!next) onClose();
  }
</script>

{#if layout.mode === 'touch'}
  <BottomSheet
    open
    onOpenChange={closeSheet}
    {title}
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
        <Dialog.Title class="flex-none text-base font-semibold text-light-950">
          {title}
        </Dialog.Title>
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
