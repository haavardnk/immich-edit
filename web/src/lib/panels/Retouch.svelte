<script lang="ts">
  import Notice from '$lib/components/Notice.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '$lib/components/editor/controls/segmentedControl';
  import { hint, keyLabel } from '$lib/keybinds';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { MAX_RETOUCH_STROKES, type RetouchMode, type RetouchStroke } from '$lib/types/edits';
  import { Button, Icon, IconButton, Tooltip } from '@immich/ui';
  import { mdiBandage, mdiClose, mdiEye, mdiEyeOff, mdiRestore, mdiStamper } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';

  const strokes = $derived(editor.edits.retouch);
  const selected = $derived<RetouchStroke | null>(
    editor.activeRetouchId ? (strokes.find((s) => s.id === editor.activeRetouchId) ?? null) : null
  );
  const mode = $derived(selected ? selected.mode : editor.retouchTool.mode);
  const size = $derived(selected ? selected.radius : editor.retouchTool.size);
  const hardness = $derived(selected ? selected.hardness : editor.retouchTool.hardness);
  const opacity = $derived(selected ? selected.opacity : editor.retouchTool.opacity);

  function setMode(m: RetouchMode): void {
    editor.setRetouchMode(m);
  }

  function live(
    patch: Partial<RetouchStroke>,
    toolPatch: Partial<typeof editor.retouchTool>
  ): void {
    if (selected) {
      void editor.setRetouchStroke(selected.id, patch, false);
      return;
    }
    editor.setRetouchTool(toolPatch);
  }

  function commit(): void {
    if (selected) void editor.commitRetouch();
  }

  function label(s: RetouchStroke, i: number): string {
    return `${s.mode === 'heal' ? 'Heal' : 'Clone'} ${i + 1}`;
  }
</script>

<div class="flex flex-col gap-1">
  <div class="flex h-8 items-center justify-between border-b border-hairline">
    <div class="text-[9px] font-semibold uppercase text-dark/65">Tool</div>
    <RadioGroup.Root
      bind:value={() => mode, (v) => setMode(v as RetouchMode)}
      orientation="horizontal"
      aria-label="Retouch tool"
      class={segmentedControlClass}
    >
      <Tooltip
        text="{hint('Heal', 'retouchHeal')} — blends texture from the source into the target"
      >
        {#snippet child({ props })}
          <RadioGroup.Item value="heal" class="{segmentedRadioItemClass} gap-1 px-2" {...props}>
            <Icon icon={mdiBandage} size="12px" aria-hidden="true" />
            Heal
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
      <Tooltip text="{hint('Clone', 'retouchClone')} — copies the source pixels exactly">
        {#snippet child({ props })}
          <RadioGroup.Item value="clone" class="{segmentedRadioItemClass} gap-1 px-2" {...props}>
            <Icon icon={mdiStamper} size="12px" aria-hidden="true" />
            Clone
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
    </RadioGroup.Root>
  </div>

  <SliderRow
    label="Size"
    value={size}
    min={0.005}
    max={0.3}
    step={0.005}
    defaultValue={0.05}
    onLive={(v: number) => live({ radius: v }, { size: v })}
    onCommit={commit}
    format={(v: number) => v.toFixed(3)}
  />
  <SliderRow
    label="Hardness"
    value={hardness}
    min={0}
    max={1}
    step={0.01}
    defaultValue={0.5}
    onLive={(v: number) => live({ hardness: v }, { hardness: v })}
    onCommit={commit}
    format={(v: number) => v.toFixed(2)}
  />
  <SliderRow
    label="Opacity"
    value={opacity}
    min={0.05}
    max={1}
    step={0.01}
    defaultValue={1}
    onLive={(v: number) => live({ opacity: v }, { opacity: v })}
    onCommit={commit}
    format={(v: number) => v.toFixed(2)}
  />

  {#if editor.retouchAnchor}
    <p class="border-l-2 border-primary/50 pl-2 text-[10px] leading-snug text-dark/65">
      Drag over a blemish to paint it out. The source follows your brush, offset from the point you
      sampled. Hold <kbd class="rounded bg-light-200 px-1 font-sans">{keyLabel('Alt')}</kbd> and click
      to sample somewhere else, or drag the green circle to move one stroke's source.
    </p>
  {:else}
    <p
      class="border-l-2 border-white/15 bg-white/4 py-1 pl-2 text-[10px] leading-snug text-dark/65"
    >
      Hold <kbd class="rounded bg-light-200 px-1 font-sans">{keyLabel('Alt')}</kbd> and click a clean
      patch of the photo to set the source. Painting is disabled until you do.
    </p>
  {/if}

  {#if strokes.length > 0}
    <div class="flex flex-col gap-0.5">
      {#each strokes as stroke, i (stroke.id)}
        <div
          class="flex h-7 items-center gap-1 rounded-md border px-1 transition-colors {editor.activeRetouchId ===
          stroke.id
            ? 'border-primary/25 bg-primary/10'
            : 'border-transparent hover:border-hairline hover:bg-white/4'}"
        >
          <Icon
            icon={stroke.mode === 'heal' ? mdiBandage : mdiStamper}
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
            {label(stroke, i)}
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
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      fullWidth
      class="h-6 panel-action"
      leadingIcon={mdiRestore}
      onclick={() => void editor.clearRetouch()}
    >
      Remove all
    </Button>
  {/if}

  {#if strokes.length >= MAX_RETOUCH_STROKES}
    <Notice color="warning" message={`Limit of ${MAX_RETOUCH_STROKES} retouch strokes reached.`} />
  {/if}
</div>
