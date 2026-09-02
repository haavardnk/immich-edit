<script lang="ts">
  import { untrack } from 'svelte';
  import { hint } from '$lib/keybinds';
  import { editor } from '$lib/stores/editor.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import ResetButton from '$lib/components/editor/controls/ResetButton.svelte';
  import SectionHeader from '$lib/components/editor/controls/SectionHeader.svelte';
  import { type AspectLock } from '$lib/types/edits';
  import {
    neutralPerspective,
    perspectiveIsIdentity,
    type PerspectiveEdits
  } from '$lib/utils/perspective';
  import { Button, Field, IconButton, Select } from '@immich/ui';
  import {
    mdiRotateLeft,
    mdiRotateRight,
    mdiFlipHorizontal,
    mdiFlipVertical,
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

  const aspectSelectOptions = aspectOptions.map((o) => ({
    value: aspectKey(o.value),
    label: o.label
  }));

  function onAspectChange(key: string): void {
    const sess = editor.geometrySession;
    if (!sess) return;
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
  const cropModified = $derived(
    !!editor.geometrySession &&
      (editor.geometrySession.userEditedCrop ||
        editor.geometrySession.draftAspect.kind !== 'original')
  );
  const transformModified = $derived(
    !!editor.geometrySession &&
      (Math.abs(editor.geometrySession.draftAngle) > 1e-4 ||
        !perspectiveIsIdentity(editor.geometrySession.draftPerspective))
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

<div class="flex flex-col divide-y divide-dark/10">
  {#if editor.geometrySession}
    <div class="flex flex-col gap-1 pb-1.5">
      <SectionHeader title="Crop" modified={cropModified} onReset={resetCrop} />
      <div class="flex items-center gap-1.5">
        <Field label="Aspect Ratio" size="tiny" class="min-w-0 flex-1">
          <Select
            size="tiny"
            class="editor-compact-select editor-compact-field"
            options={aspectSelectOptions}
            value={aspectKey(editor.geometrySession.draftAspect)}
            onChange={onAspectChange}
          />
        </Field>
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          class="size-7 bg-neutral-800 not-disabled:hover:bg-neutral-700"
          icon={isPortrait ? mdiCropPortrait : mdiCropLandscape}
          title={isPortrait ? 'Switch to landscape' : 'Switch to portrait'}
          aria-label={isPortrait ? 'Switch to landscape' : 'Switch to portrait'}
          disabled={!orientationAvailable}
          onclick={toggleOrientation}
        />
      </div>
    </div>

    <div class="flex flex-col gap-1 py-1.5">
      <div class="flex h-6 items-center justify-between border-b border-white/6">
        <div class="text-[9px] font-semibold uppercase text-dark/65">Transform</div>
        <div class="flex items-center gap-1">
          <IconButton
            size="tiny"
            variant="ghost"
            color={ui.perspectiveCorners ? 'primary' : 'secondary'}
            icon={mdiVectorSquare}
            title={hint('Corner handles', 'perspective')}
            aria-label="Corner handles"
            aria-pressed={ui.perspectiveCorners}
            onclick={ui.togglePerspectiveCorners}
          />
          <ResetButton
            title="Reset Transform"
            disabled={!transformModified}
            onclick={resetTransform}
          />
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

    <div class="grid grid-cols-2 gap-1 border-t border-hairline pt-1.5">
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 panel-action"
        leadingIcon={mdiRotateLeft}
        aria-label="Rotate left 90°"
        onclick={rotateLeft}
      >
        90°
      </Button>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="h-7 panel-action"
        leadingIcon={mdiRotateRight}
        aria-label="Rotate right 90°"
        onclick={rotateRight}
      >
        90°
      </Button>
      <Button
        size="tiny"
        variant={(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h)
          ? 'filled'
          : 'ghost'}
        color={(editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h)
          ? 'primary'
          : 'secondary'}
        class="h-7 panel-action"
        leadingIcon={mdiFlipHorizontal}
        aria-pressed={editor.geometrySession?.draftFlipH ?? editor.edits.geometry.flip_h}
        onclick={toggleFlipH}
      >
        Flip Horizontal
      </Button>
      <Button
        size="tiny"
        variant={(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v)
          ? 'filled'
          : 'ghost'}
        color={(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v)
          ? 'primary'
          : 'secondary'}
        class="h-7 panel-action"
        leadingIcon={mdiFlipVertical}
        aria-pressed={editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v}
        onclick={toggleFlipV}
      >
        Flip Vertical
      </Button>
    </div>
  {/if}
</div>
