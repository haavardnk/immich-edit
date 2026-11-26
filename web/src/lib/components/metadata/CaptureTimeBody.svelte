<script lang="ts">
  import ChipChoice from '#lib/components/chips/ChipChoice.svelte';
  import CameraPick from '#lib/components/metadata/CameraPick.svelte';
  import CaptureByPhoto from '#lib/components/metadata/CaptureByPhoto.svelte';
  import CaptureFromName from '#lib/components/metadata/CaptureFromName.svelte';
  import CaptureOnce from '#lib/components/metadata/CaptureOnce.svelte';
  import CapturePreview from '#lib/components/metadata/CapturePreview.svelte';
  import CaptureZones from '#lib/components/metadata/CaptureZones.svelte';
  import ClockStepper from '#lib/components/metadata/ClockStepper.svelte';
  import {
    ALL_CAMERAS,
    SHIFT_LIMIT,
    captureEditor,
    type CaptureMode
  } from '#lib/stores/captureEditor.svelte.ts';

  const editor = captureEditor;

  let { width = $bindable(0) }: { width?: number } = $props();

  const MODES: { value: CaptureMode; label: string; several?: boolean }[] = [
    { value: 'shift', label: 'Shift' },
    { value: 'photo', label: 'Set by a photo', several: true },
    { value: 'once', label: 'One time for all', several: true },
    { value: 'name', label: 'From file name' }
  ];

  const modes = $derived(editor.photos.length > 1 ? MODES : MODES.filter((mode) => !mode.several));
</script>

<div class="@container flex min-h-0 flex-1 flex-col" bind:clientWidth={width}>
  <div class="flex min-h-0 flex-1 flex-col @3xl:h-[min(40rem,70vh)] @3xl:flex-none @3xl:flex-row">
    <section
      class="flex shrink-0 flex-col gap-3 px-5 py-4 @3xl:min-w-[27rem] @3xl:overflow-y-auto @3xl:border-e @3xl:border-hairline touch:px-0 touch:pt-0 touch:@3xl:pe-4"
      aria-label="Change"
    >
      <div class="@3xl:w-max">
        <ChipChoice
          options={modes}
          value={editor.mode}
          label="How to change the capture time"
          onChange={(mode) => (editor.mode = mode)}
        />
      </div>
      <div class="flex flex-col gap-3 contain-inline-size @3xl:min-h-0 @3xl:flex-1">
        {#if editor.perCamera}
          <CameraPick
            cameras={editor.cameras}
            chosen={editor.camera}
            all={{ key: ALL_CAMERAS, count: editor.photos.length }}
            detail={editor.shiftTextOf}
            onChoose={editor.chooseCamera}
          />
        {/if}
        {#if editor.mode === 'shift'}
          <ClockStepper
            label="Shift by"
            name="Shift by"
            value={editor.correction}
            limit={SHIFT_LIMIT}
            onChange={(ms) => (editor.correction = ms)}
          />
        {:else if editor.mode === 'photo'}
          <CaptureByPhoto />
        {:else if editor.mode === 'once'}
          <CaptureOnce />
        {:else}
          <CaptureFromName />
        {/if}
        <CaptureZones />
      </div>
    </section>
    <CapturePreview />
  </div>
</div>
