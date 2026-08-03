<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { MAX_RETOUCH_STROKES, type RetouchMode, type RetouchStroke } from '$lib/types/edits';
  import { mdiBandage, mdiClose, mdiEye, mdiEyeOff, mdiRestore, mdiStamper } from '@mdi/js';

  const strokes = $derived(editor.edits.retouch);
  const selected = $derived<RetouchStroke | null>(
    editor.activeRetouchId ? strokes.find((s) => s.id === editor.activeRetouchId) ?? null : null
  );
  const mode = $derived(selected ? selected.mode : editor.retouchTool.mode);
  const size = $derived(selected ? selected.radius : editor.retouchTool.size);
  const hardness = $derived(selected ? selected.hardness : editor.retouchTool.hardness);
  const opacity = $derived(selected ? selected.opacity : editor.retouchTool.opacity);

  function setMode(m: RetouchMode): void {
    editor.setRetouchMode(m);
  }

  function live(patch: Partial<RetouchStroke>, toolPatch: Partial<typeof editor.retouchTool>): void {
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

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-between">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Tool</div>
    <div class="flex rounded ring-1 ring-white/10 overflow-hidden text-[10px]">
      <button
        type="button"
        class="flex items-center gap-1 px-2 leading-5 transition-colors {mode === 'heal'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        title="Heal (H) — blends texture from the source into the target"        onclick={() => setMode('heal')}
      >
        <Icon path={mdiBandage} size={12} />
        Heal
      </button>
      <button
        type="button"
        class="flex items-center gap-1 px-2 leading-5 transition-colors {mode === 'clone'
          ? 'bg-white/15 text-immich-dark-fg'
          : 'text-immich-dark-fg/50 hover:text-immich-dark-fg'}"
        title="Clone (C) — copies the source pixels exactly"
        onclick={() => setMode('clone')}
      >
        <Icon path={mdiStamper} size={12} />
        Clone
      </button>
    </div>
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
    <p class="text-[11px] text-immich-dark-fg/40 leading-snug">
      Drag over a blemish to paint it out. The source follows your brush, offset from the point you
      sampled. Hold <kbd class="px-1 rounded bg-white/10 font-sans">Alt</kbd> and click to sample
      somewhere else, or drag the green circle to move one stroke's source.
    </p>
  {:else}
    <p
      class="text-[11px] text-immich-dark-primary/90 leading-snug rounded-lg bg-immich-dark-primary/10 px-2 py-1.5"
    >
      Hold <kbd class="px-1 rounded bg-white/10 font-sans">Alt</kbd> and click a clean patch of the
      photo to set the source. Painting is disabled until you do.
    </p>
  {/if}

  {#if strokes.length > 0}
    <div class="flex flex-col gap-0.5">
      {#each strokes as stroke, i (stroke.id)}
        <div
          class="flex items-center gap-1.5 px-1.5 py-1 rounded cursor-pointer transition-colors {editor.activeRetouchId ===
          stroke.id
            ? 'bg-white/10'
            : 'hover:bg-white/5'}"
          role="button"
          tabindex="0"
          onclick={() => (editor.activeRetouchId = stroke.id)}
          onkeydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') editor.activeRetouchId = stroke.id;
          }}
        >
          <Icon
            path={stroke.mode === 'heal' ? mdiBandage : mdiStamper}
            size={13}
            class="shrink-0 opacity-50"
          />
          <span class="flex-1 text-xs text-immich-dark-fg/90 truncate {stroke.enabled ? '' : 'opacity-50'}">
            {label(stroke, i)}
          </span>
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/50 hover:text-immich-dark-fg"
            title={stroke.enabled ? 'Disable' : 'Enable'}
            aria-label="Toggle retouch stroke"
            onclick={(e) => {
              e.stopPropagation();
              void editor.toggleRetouchStroke(stroke.id);
            }}
          >
            <Icon path={stroke.enabled ? mdiEye : mdiEyeOff} size={13} />
          </button>
          <button
            type="button"
            class="shrink-0 text-immich-dark-fg/40 hover:text-red-400 transition-colors"
            title="Delete"
            aria-label="Delete retouch stroke"
            onclick={(e) => {
              e.stopPropagation();
              void editor.removeRetouchStroke(stroke.id);
            }}
          >
            <Icon path={mdiClose} size={13} />
          </button>
        </div>
      {/each}
    </div>
    <button
      type="button"
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
      onclick={() => void editor.clearRetouch()}
    >
      <Icon path={mdiRestore} size={14} />
      Remove all
    </button>
  {/if}

  {#if strokes.length >= MAX_RETOUCH_STROKES}
    <div class="text-[11px] text-amber-400/80">
      Limit of {MAX_RETOUCH_STROKES} retouch strokes reached.
    </div>
  {/if}
</div>
