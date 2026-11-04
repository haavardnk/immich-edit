<script lang="ts">
  import { EXTENSION_BY_FORMAT } from '#lib/api/export.ts';
  import DestinationToggle from '#lib/panels/export/DestinationToggle.svelte';
  import ExportPresetMenu from '#lib/panels/export/ExportPresetMenu.svelte';
  import ExportSection from '#lib/panels/export/ExportSection.svelte';
  import FilenameTemplateField from '#lib/panels/export/FilenameTemplateField.svelte';
  import FormatOptions from '#lib/panels/export/FormatOptions.svelte';
  import ResizeOptions from '#lib/panels/export/ResizeOptions.svelte';
  import SharpenOptions from '#lib/panels/export/SharpenOptions.svelte';
  import WatermarkOptions from '#lib/panels/export/WatermarkOptions.svelte';
  import ImmichOptions from '#lib/panels/export/ImmichOptions.svelte';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';
  import { ensureLibraryLoaded } from '#lib/panels/export/settings.ts';
  import type { ExportScope } from '#lib/panels/export/scope.ts';
  import type { PixelSize } from '#lib/panels/export/resize.ts';
  import type { Snippet } from 'svelte';

  let {
    scope,
    replaceOriginal = $bindable(false),
    crop = null,
    downloadLabel,
    fileNotice
  }: {
    scope: ExportScope;
    replaceOriginal?: boolean;
    crop?: PixelSize | null;
    downloadLabel?: string;
    fileNotice?: Snippet;
  } = $props();

  const form = $derived(exportSettings.form);
  const destination = $derived(exportSettings.destination);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded(exportSettings.form);
  });
</script>

<ExportPresetMenu />
<DestinationToggle bind:value={exportSettings.destination} {downloadLabel} />

<ExportSection title="File">
  <FormatOptions bind:form={exportSettings.form} />
  {@render fileNotice?.()}
</ExportSection>

<ExportSection title="Size">
  <ResizeOptions bind:form={exportSettings.form} {crop} />
  <SharpenOptions bind:form={exportSettings.form} />
</ExportSection>

<ExportSection title="Watermark">
  <WatermarkOptions bind:form={exportSettings.form} />
</ExportSection>

<ExportSection title="Name">
  <FilenameTemplateField
    bind:value={exportSettings.form.filenameTemplate}
    example={scope.nameExample}
    extension={EXTENSION_BY_FORMAT[form.format]}
  />
</ExportSection>

{#if destination === 'immich'}
  <ExportSection title="Immich">
    <ImmichOptions
      bind:form={exportSettings.form}
      bind:replaceOriginal
      stackable={scope.stackable}
      stackReason={scope.stackReason}
      replaceReason={scope.replaceReason}
    />
  </ExportSection>
{/if}
