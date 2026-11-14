<script lang="ts">
  import { RETOUCH_MODE_ICON } from '#lib/editor/retouch/modes.ts';
  import { strokeCount, strokeTitle } from '#lib/editor/retouch/text.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiClose, mdiEye, mdiEyeOff, mdiRestore } from '@mdi/js';

  const strokes = $derived(editor.edits.retouch);
</script>

<div class="flex h-7 items-center justify-between gap-2">
  <span class="text-[10px] font-medium text-dark/65 uppercase">
    Strokes <span class="tabular-nums">({strokeCount(strokes.length)})</span>
  </span>
  {#if strokes.length > 0}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="danger"
      class="h-6 panel-action"
      leadingIcon={mdiRestore}
      onclick={() => void editor.clearRetouch()}
    >
      Remove all
    </Button>
  {/if}
</div>
{#if strokes.length > 0}
  <div class="flex flex-col gap-0.5">
    {#each strokes as stroke, i (stroke.id)}
      <div
        class="flex h-7 items-center gap-1 rounded-md border px-1 transition-colors {editor.activeRetouchId ===
        stroke.id
          ? 'border-primary/25 bg-primary/10'
          : 'border-transparent hover:border-hairline hover:bg-light-950/4'}"
      >
        <Icon
          icon={RETOUCH_MODE_ICON[stroke.mode]}
          size="13px"
          class="shrink-0 opacity-50"
          aria-hidden="true"
        />
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-5 min-w-0 flex-1 justify-start truncate rounded-sm px-1 py-0 text-left text-[11px] text-dark hover:bg-transparent {stroke.enabled
            ? ''
            : 'opacity-50'}"
          aria-pressed={editor.activeRetouchId === stroke.id}
          onclick={() => (editor.activeRetouchId = stroke.id)}
        >
          {strokeTitle(stroke.mode, i)}
        </Button>
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={stroke.enabled ? mdiEye : mdiEyeOff}
          title={stroke.enabled ? 'Disable' : 'Enable'}
          aria-label="Toggle retouch stroke"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            void editor.toggleRetouchStroke(stroke.id);
          }}
        />
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiClose}
          title="Delete"
          aria-label="Delete retouch stroke"
          onclick={(e: MouseEvent) => {
            e.stopPropagation();
            void editor.removeRetouchStroke(stroke.id);
          }}
        />
      </div>
    {/each}
  </div>
{/if}
