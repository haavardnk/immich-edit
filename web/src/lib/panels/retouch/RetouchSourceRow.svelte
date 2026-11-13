<script lang="ts">
  import { layout } from '#lib/device/layout.svelte.ts';
  import { SOURCE_MISSING_TEXT } from '#lib/editor/retouch/text.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiCrosshairsGps } from '@mdi/js';

  const touch = $derived(layout.mode === 'touch');
  const missed = $derived(editor.retouchSourceMissed && !editor.retouchAnchor);
</script>

<div class="flex h-7 items-center gap-1.5 touch:h-12 touch:gap-2.5">
  <span class="editor-compact-label select-none touch:text-[14px] touch:text-dark">Source</span>
  <span
    class="rounded-full px-1.5 text-[10px] leading-4 touch:px-2 touch:py-0.5 touch:text-[12px] {editor.retouchAnchor
      ? 'bg-success-500/15 text-success-800'
      : 'bg-danger-500/15 text-danger-800'}"
  >
    {editor.retouchAnchor ? 'Set' : 'Not set'}
  </span>
  {#if touch}
    <Button
      type="button"
      size="medium"
      variant={editor.retouchSampling ? 'outline' : 'ghost'}
      color={editor.retouchSampling ? 'success' : 'secondary'}
      class="ml-auto h-10 rounded-xl px-3.5 text-[14px] font-normal {editor.retouchSampling
        ? ''
        : 'bg-ghost'}"
      leadingIcon={mdiCrosshairsGps}
      aria-pressed={editor.retouchSampling}
      onclick={editor.toggleRetouchSampling}
    >
      Set source
    </Button>
  {:else}
    <div class="ml-auto flex items-center gap-0.5">
      <Button
        type="button"
        size="tiny"
        variant={editor.retouchSampling ? 'filled' : 'ghost'}
        color={editor.retouchSampling ? 'primary' : 'secondary'}
        class="h-6 panel-action"
        title={hint('Pick the source with the next click', 'retouchSample')}
        aria-pressed={editor.retouchSampling}
        onclick={editor.toggleRetouchSampling}
      >
        Sample
      </Button>
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        title="Clear source"
        aria-label="Clear source"
        disabled={!editor.retouchAnchor}
        onclick={editor.clearRetouchAnchor}
      />
    </div>
  {/if}
</div>
{#if !touch}
  <div
    role="status"
    aria-live="polite"
    class={missed ? 'animate-pulse text-[10px] text-danger-800' : 'sr-only'}
  >
    {missed ? SOURCE_MISSING_TEXT : ''}
  </div>
{/if}
