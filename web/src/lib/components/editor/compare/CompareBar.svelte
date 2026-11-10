<script lang="ts">
  import ChipChoice from '#lib/components/chips/ChipChoice.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import CompareSourceMenu from './CompareSourceMenu.svelte';

  type Shown = 'split' | 'side';

  const touch = $derived(layout.mode === 'touch');
  const options = $derived<{ value: Shown; label: string }[]>(
    editor.compare.sideBySide
      ? [
          { value: 'split', label: 'Split' },
          { value: 'side', label: 'Side by side' }
        ]
      : [{ value: 'split', label: 'Split' }]
  );
  const mode = $derived<Shown>(editor.compare.mode === 'side' ? 'side' : 'split');
</script>

<div
  role="group"
  aria-label="Compare"
  class="relative flex h-9 shrink-0 items-center gap-2 bg-editor-chrome ps-3 pe-1.5 text-xs text-dark/65 before:absolute before:inset-x-0 before:top-0 before:h-px before:bg-hairline touch:h-14 touch:gap-2 touch:ps-4 touch:pe-2 touch:text-sm"
>
  <span class="shrink-0">Compare:</span>
  <ChipChoice
    {options}
    value={mode}
    label="Compare mode"
    onChange={(next) => editor.compare.setMode(next)}
  />
  <div class="flex min-w-0 flex-1 justify-center">
    <CompareSourceMenu />
  </div>
  {#if editor.compare.mode === 'side' && !touch}
    <span class="shrink-0">Synced zoom</span>
  {/if}
  <IconButton
    size={touch ? 'medium' : 'small'}
    variant="ghost"
    color="secondary"
    shape="round"
    icon={mdiClose}
    title="Close compare"
    aria-label="Close compare"
    onclick={() => editor.compare.end()}
  />
</div>
