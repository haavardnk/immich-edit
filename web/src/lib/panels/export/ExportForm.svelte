<script lang="ts">
  import { EXTENSION_BY_FORMAT } from '#lib/api/export.ts';
  import DestinationToggle from '#lib/panels/export/DestinationToggle.svelte';
  import ExportSection from '#lib/panels/export/ExportSection.svelte';
  import FilenameTemplateField from '#lib/panels/export/FilenameTemplateField.svelte';
  import FormatOptions from '#lib/panels/export/FormatOptions.svelte';
  import ResizeOptions from '#lib/panels/export/ResizeOptions.svelte';
  import SharpenOptions from '#lib/panels/export/SharpenOptions.svelte';
  import WatermarkOptions from '#lib/panels/export/WatermarkOptions.svelte';
  import ImmichOptions from '#lib/panels/export/ImmichOptions.svelte';
  import OriginalName from '#lib/panels/export/OriginalName.svelte';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { ensureLibraryLoaded } from '#lib/panels/export/settings.ts';
  import type { ExportScope } from '#lib/panels/export/scope.ts';
  import type { PixelSize } from '#lib/panels/export/resize.ts';
  import type { Snippet } from 'svelte';
  import {
    fileSummary,
    nameSummary,
    sizeSummary,
    watermarkSummary
  } from '#lib/panels/export/summary.ts';
  import { watermarkLibrary } from '#lib/panels/export/watermarkLibrary.svelte.ts';

  let {
    scope,
    replaceOriginal = $bindable(false),
    crop = null,
    onWatermarkPick,
    fileNotice
  }: {
    scope: ExportScope;
    replaceOriginal?: boolean;
    crop?: PixelSize | null;
    onWatermarkPick?: () => void;
    fileNotice?: Snippet;
  } = $props();

  const form = $derived(exportSettings.form);
  const destination = $derived(exportSettings.destination);
  const keepsName = $derived(
    destination === 'immich' && replaceOriginal && scope.replaceReason === null
  );

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded(exportSettings.form);
  });
</script>

<DestinationToggle bind:value={exportSettings.destination} />

<ExportSection title="File" summary={fileSummary(form)}>
  <FormatOptions bind:form={exportSettings.form} />
  {@render fileNotice?.()}
</ExportSection>

<ExportSection title="Size" summary={sizeSummary(form, crop)}>
  <ResizeOptions bind:form={exportSettings.form} {crop} />
  <SharpenOptions bind:form={exportSettings.form} />
</ExportSection>

<ExportSection
  title="Watermark"
  summary={watermarkSummary(form, watermarkLibrary.nameOf(form.watermarkId))}
>
  <WatermarkOptions bind:form={exportSettings.form} onpick={onWatermarkPick} />
</ExportSection>

<ExportSection
  title="Name"
  summary={keepsName ? 'Original name' : nameSummary(form)}
  mono={!keepsName}
>
  {#if keepsName}
    <OriginalName example={scope.nameExample} extension={EXTENSION_BY_FORMAT[form.format]} />
  {:else}
    <FilenameTemplateField
      bind:value={exportSettings.form.filenameTemplate}
      example={scope.nameExample}
      extension={EXTENSION_BY_FORMAT[form.format]}
    />
  {/if}
</ExportSection>

{#if destination === 'immich'}
  <ExportSection title="Immich" foldable={false}>
    <ImmichOptions
      bind:form={exportSettings.form}
      bind:replaceOriginal
      stackable={scope.stackable}
      stackReason={scope.stackReason}
      replaceReason={scope.replaceReason}
    />
  </ExportSection>
{/if}
