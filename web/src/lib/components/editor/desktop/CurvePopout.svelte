<script lang="ts">
  import CurvesPanel from '#lib/panels/Curves.svelte';
  import { curveEditor } from '#lib/stores/curveEditor.svelte.ts';
  import { holdEscape } from '#lib/stores/overlays.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { Dialog } from 'bits-ui';

  $effect(() => {
    if (curveEditor.poppedOut) return holdEscape();
  });
</script>

<Dialog.Root bind:open={() => curveEditor.poppedOut, (open) => (curveEditor.poppedOut = open)}>
  <Dialog.Portal>
    <Dialog.Content
      class="fixed top-1/2 left-[calc(50%-11rem)] z-50 flex w-[min(44rem,calc(100%-26rem))] -translate-x-1/2 -translate-y-1/2 flex-col gap-2 rounded-xl border border-hairline bg-editor-panel p-3 shadow-2xl"
    >
      <div class="flex items-center justify-between">
        <Dialog.Title class="text-sm font-semibold text-light-950">Curves</Dialog.Title>
        <Dialog.Close>
          {#snippet child({ props })}
            <IconButton
              size="small"
              variant="ghost"
              color="secondary"
              shape="round"
              icon={mdiClose}
              title="Close"
              aria-label="Close curve editor"
              {...props}
            />
          {/snippet}
        </Dialog.Close>
      </div>
      <CurvesPanel />
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
