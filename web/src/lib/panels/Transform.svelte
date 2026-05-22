<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { type AspectLock } from '$lib/types/edits';
  import {
    mdiRotateLeft,
    mdiRotateRight,
    mdiFlipHorizontal,
    mdiFlipVertical,
    mdiRestore,
    mdiCropLandscape,
    mdiCropPortrait
  } from '@mdi/js';

  onMount(() => {
    editor.enterCropMode();
    return () => {
      void editor.exitCropMode();
    };
  });

  function rotateLeft(): void {
    editor.edits.geometry.rotate = ((editor.edits.geometry.rotate + 270) % 360) as 0 | 90 | 180 | 270;
    void editor.onCommit();
  }
  function rotateRight(): void {
    editor.edits.geometry.rotate = ((editor.edits.geometry.rotate + 90) % 360) as 0 | 90 | 180 | 270;
    void editor.onCommit();
  }
  function toggleFlipH(): void {
    editor.edits.geometry.flip_h = !editor.edits.geometry.flip_h;
    void editor.onCommit();
  }
  function toggleFlipV(): void {
    editor.edits.geometry.flip_v = !editor.edits.geometry.flip_v;
    void editor.onCommit();
  }

  function reset(): void {
    editor.resetCropDraft();
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
    const key = (e.currentTarget as HTMLSelectElement).value;
    const opt = aspectOptions.find((o) => aspectKey(o.value) === key);
    if (!opt) return;
    if (opt.value.kind === 'ratio' && editor.cropSession) {
      const cur = editor.cropSession.draftAspect;
      const wantPortrait = cur.kind === 'ratio' && cur.num < cur.den;
      const next: AspectLock = wantPortrait
        ? { kind: 'ratio', num: opt.value.den, den: opt.value.num }
        : opt.value;
      editor.updateCropDraftAspect(next);
    } else {
      editor.updateCropDraftAspect(opt.value);
    }
  }

  const isPortrait = $derived(
    editor.cropSession?.draftAspect.kind === 'ratio' &&
      editor.cropSession.draftAspect.num < editor.cropSession.draftAspect.den
  );
  const orientationAvailable = $derived(
    editor.cropSession?.draftAspect.kind === 'ratio' &&
      editor.cropSession.draftAspect.num !== editor.cropSession.draftAspect.den
  );

  function toggleOrientation(): void {
    const sess = editor.cropSession;
    if (!sess || sess.draftAspect.kind !== 'ratio') return;
    editor.updateCropDraftAspect({
      kind: 'ratio',
      num: sess.draftAspect.den,
      den: sess.draftAspect.num
    });
  }

</script>

<div class="flex flex-col gap-3">
  {#if editor.cropSession}
    <div class="flex flex-col gap-2 text-xs">
      <label class="flex flex-col gap-1">
        <span class="flex justify-between"><span>Angle</span><span class="opacity-60">{editor.cropSession.draftAngle.toFixed(1)}°</span></span>
        <input
          type="range"
          min="-45"
          max="45"
          step="0.1"
          value={editor.cropSession.draftAngle}
          oninput={(e) => editor.updateCropDraftAngle(parseFloat((e.currentTarget as HTMLInputElement).value))}
          class="range range-xs"
        />
      </label>
      <label class="flex flex-col gap-1">
        <span>Aspect Ratio</span>
        <div class="flex gap-1.5 items-center">
          <select
            class="select bg-white/5 flex-1 rounded-lg text-xs h-auto py-1.5 min-h-0"
            value={aspectKey(editor.cropSession.draftAspect)}
            onchange={onAspectChange}
          >
            {#each aspectOptions as o}
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
      </label>
      <button class="flex items-center justify-center gap-1 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs" onclick={reset}>
        <Icon path={mdiRestore} size={14} /> Reset crop
      </button>
    </div>
  {/if}
  <div class="grid grid-cols-2 gap-1.5">
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
        onclick={rotateLeft}
      >
        <Icon path={mdiRotateLeft} size={16} />
        90°
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
        onclick={rotateRight}
      >
        <Icon path={mdiRotateRight} size={16} />
        90°
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {editor.edits.geometry.flip_h ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
        onclick={toggleFlipH}
      >
        <Icon path={mdiFlipHorizontal} size={16} />
        Flip Horizontal
      </button>
      <button
        class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg transition-colors text-xs {editor.edits.geometry.flip_v ? 'bg-immich-dark-primary/20 text-immich-dark-primary' : 'bg-white/5 hover:bg-white/10'}"
        onclick={toggleFlipV}
      >
        <Icon path={mdiFlipVertical} size={16} />
        Flip Vertical
      </button>
    </div>
</div>
