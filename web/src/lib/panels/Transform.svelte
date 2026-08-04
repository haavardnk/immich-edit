<script lang="ts">
  import { untrack } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { type AspectLock } from '$lib/types/edits';
  import { neutralPerspective, type PerspectiveEdits } from '$lib/utils/perspective';
  import {
    mdiRotateLeft,
    mdiRotateRight,
    mdiFlipHorizontal,
    mdiFlipVertical,
    mdiRestore,
    mdiCropLandscape,
    mdiCropPortrait,
    mdiVectorSquare
  } from '@mdi/js';

  $effect(() => {
    const assetId = editor.assetId;
    const initialised = editor.initialised;
    if (!assetId || !initialised) return;
    untrack(() => editor.startGeometrySession());
    return () => {
      untrack(() => void editor.finishGeometrySession());
    };
  });

  function rotateLeft(): void {
    editor.rotateStep(270);
  }
  function rotateRight(): void {
    editor.rotateStep(90);
  }
  function toggleFlipH(): void {
    editor.flipStep('h');
  }
  function toggleFlipV(): void {
    editor.flipStep('v');
  }

  function resetCrop(): void {
    editor.updateGeometryDraftAspect({ kind: 'original' });
  }

  const aspectOptions: Array<{ label: string; value: AspectLock }> = [
    { label: 'Original', value: { kind: 'original' } },
    { label: 'Free', value: { kind: 'free' } },
    { label: '1:1', value: { kind: 'ratio', num: 1, den: 1 } },
    { label: '3:2', value: { kind: 'ratio', num: 3, den: 2 } },
    { label: '4:3', value: { kind: 'ratio', num: 4, den: 3 } },
    { label: '16:9', value: { kind: 'ratio', num: 16, den: 9 } }
  ];

  function aspectKey(a: AspectLock): string {
    if (a.kind === 'ratio') {
      const lo = Math.min(a.num, a.den);
      const hi = Math.max(a.num, a.den);
      return `r-${hi}-${lo}`;
    }
    return a.kind;
  }
  function onAspectChange(e: Event): void {
    const sess = editor.geometrySession;
    if (!sess) return;
    const key = (e.currentTarget as HTMLSelectElement).value;
    const opt = aspectOptions.find((o) => aspectKey(o.value) === key);
    if (!opt) return;
    if (opt.value.kind === 'ratio') {
      const cur = sess.draftAspect;
      const wantPortrait = cur.kind === 'ratio' && cur.num < cur.den;
      const next: AspectLock = wantPortrait
        ? { kind: 'ratio', num: opt.value.den, den: opt.value.num }
        : opt.value;
      editor.updateGeometryDraftAspect(next);
    } else {
      editor.updateGeometryDraftAspect(opt.value);
    }
  }

  const isPortrait = $derived(
    editor.geometrySession?.draftAspect.kind === 'ratio' &&
      editor.geometrySession.draftAspect.num < editor.geometrySession.draftAspect.den
  );
  const orientationAvailable = $derived(
    editor.geometrySession?.draftAspect.kind === 'ratio' &&
      editor.geometrySession.draftAspect.num !== editor.geometrySession.draftAspect.den
  );

  function toggleOrientation(): void {
    const sess = editor.geometrySession;
    if (!sess || sess.draftAspect.kind !== 'ratio') return;
    editor.updateGeometryDraftAspect({
      kind: 'ratio',
      num: sess.draftAspect.den,
      den: sess.draftAspect.num
    });
  }

  const perspectiveSliders: Array<{
    key: keyof PerspectiveEdits & ('vertical' | 'horizontal' | 'aspect');
    label: string;
  }> = [
    { key: 'vertical', label: 'Vertical' },
    { key: 'horizontal', label: 'Horizontal' },
    { key: 'aspect', label: 'Aspect' }
  ];

  function onPerspectiveLive(key: (typeof perspectiveSliders)[number]['key'], v: number): void {
    editor.updateGeometryDraftPerspective({ [key]: v });
  }

  function resetTransform(): void {
    editor.updateGeometryDraftAngle(0);
    editor.updateGeometryDraftPerspective(neutralPerspective());
  }

  function noCommit(): void {}
</script>

<div class="flex flex-col divide-y divide-white/5">
  {#if editor.geometrySession}
    <div class="flex flex-col gap-2.5 pb-3">
      <div class="flex items-center justify-between">
        <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Crop</div>
        <button
          type="button"
          class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
          title="Reset Crop"
          aria-label="Reset Crop"
          onclick={resetCrop}
        >
          <Icon path={mdiRestore} size={14} />
        </button>
      </div>
      <div class="flex gap-1.5 items-center">
        <select
          aria-label="Aspect Ratio"
          class="select bg-white/5 flex-1 rounded-lg text-xs h-auto py-1.5 min-h-0"
          value={aspectKey(editor.geometrySession.draftAspect)}
          onchange={onAspectChange}
        >
          {#each aspectOptions as o (aspectKey(o.value))}
            <option value={aspectKey(o.value)}>{o.label}</option>
          {/each}
        </select>
        <button
          type="button"
          class="p-1.5 rounded-lg text-xs transition-colors {orientationAvailable ? 'bg-white/5 hover:bg-white/10' : 'bg-white/5 opacity-40 cursor-not-allowed'}"
          onclick={toggleOrientation}
          disabled={!orientationAvailable}
          aria-label={isPortrait ? 'Switch to landscape' : 'Switch to portrait'}
          title={isPortrait ? 'Switch to landscape' : 'Switch to portrait'}
        >
          <Icon path={isPortrait ? mdiCropPortrait : mdiCropLandscape} size={16} />
        </button>
      </div>
    </div>

    <div class="flex flex-col gap-2.5 py-3">
      <div class="flex items-center justify-between">
        <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Transform</div>
        <div class="flex items-center gap-2">
          <button
            type="button"
            class="transition-colors {ui.perspectiveCorners
              ? 'text-immich-dark-primary'
              : 'text-immich-dark-fg/40 hover:text-immich-dark-fg'}"
            aria-pressed={ui.perspectiveCorners}
            onclick={ui.togglePerspectiveCorners}
            aria-label="Corner handles"
            title="Corner handles (P)"
          >
            <Icon path={mdiVectorSquare} size={14} />
          </button>
          <button
            type="button"
            class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
            title="Reset Transform"
            aria-label="Reset Transform"
            onclick={resetTransform}
          >
            <Icon path={mdiRestore} size={14} />
          </button>
        </div>
      </div>
      <SliderRow
        label="Angle"
        value={editor.geometrySession.draftAngle}
        min={-45}
        max={45}
        step={0.1}
        onLive={editor.updateGeometryDraftAngle}
        onCommit={noCommit}
        format={(v: number) => `${v.toFixed(1)}°`}
      />
      {#each perspectiveSliders as s (s.key)}
        <SliderRow
          label={s.label}
          value={editor.geometrySession.draftPerspective[s.key]}
          min={-100}
          max={100}
          step={1}
          onLive={(v: number) => onPerspectiveLive(s.key, v)}
          onCommit={noCommit}
          format={(v: number) => v.toFixed(0)}
        />
      {/each}
    </div>

    <div class="grid grid-cols-2 gap-1.5 pt-3">
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
        aria-label="Rotate left 90°"
        onclick={rotateLeft}
      >
        <Icon path={mdiRotateLeft} size={16} />
        90°
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
        aria-label="Rotate right 90°"
        onclick={rotateRight}
      >
        <Icon path={mdiRotateRight} size={16} />
        90°
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h) ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
        aria-pressed={editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h}
        onclick={toggleFlipH}
      >
        <Icon path={mdiFlipHorizontal} size={16} />
        Flip Horizontal
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v) ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
        aria-pressed={editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v}
        onclick={toggleFlipV}
      >
        <Icon path={mdiFlipVertical} size={16} />
        Flip Vertical
      </button>
    </div>
  {/if}
</div>
