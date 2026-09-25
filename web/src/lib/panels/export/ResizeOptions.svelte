<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { Field, Select } from '@immich/ui';
  import {
    EXPORT_MAX_EDGE,
    MAX_RESIZE_MEGAPIXELS,
    MAX_RESIZE_PERCENT,
    linkedEdge,
    resizeError,
    type PixelSize
  } from './resize';
  import { RESIZE_MODES, formResize, type ExportForm } from './settings';

  let {
    form = $bindable<ExportForm>(),
    crop = null
  }: { form: ExportForm; crop?: PixelSize | null } = $props();

  const error = $derived(resizeError(formResize(form)));
  const edgeInputClass =
    'ring-0 focus-within:ring-1 focus-within:ring-primary [&_input]:h-7 [&_input]:px-2';

  function readNumber(event: Event): number | null {
    const value = (event.currentTarget as HTMLInputElement).valueAsNumber;
    return Number.isNaN(value) ? null : value;
  }

  function setWidth(width: number | null): void {
    form.resizeWidth = width;
    if (crop && width !== null) form.resizeHeight = linkedEdge(width, crop.w, crop.h);
  }

  function setHeight(height: number | null): void {
    form.resizeHeight = height;
    if (crop && height !== null) form.resizeWidth = linkedEdge(height, crop.h, crop.w);
  }
</script>

<Field label="Resize" size="tiny">
  <Select
    size="tiny"
    class="editor-compact-select editor-compact-field"
    options={RESIZE_MODES}
    value={form.resizeMode}
    onChange={(mode) => {
      form.resizeMode = mode;
      if (mode === 'dimensions' && crop) {
        form.resizeWidth = crop.w;
        form.resizeHeight = crop.h;
      }
    }}
  />
</Field>

{#if form.resizeMode === 'dimensions'}
  <div class="panel-row h-7 items-center">
    <span class="editor-compact-label select-none">Pixels</span>
    <div class="col-span-2 grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-1">
      <TextInput
        type="number"
        compact
        color="neutral"
        class={edgeInputClass}
        inputSize={1}
        aria-label="Width"
        placeholder="Width"
        min={1}
        max={EXPORT_MAX_EDGE}
        step={1}
        value={form.resizeWidth === null ? '' : String(form.resizeWidth)}
        aria-invalid={error ? 'true' : undefined}
        oninput={(event: Event) => setWidth(readNumber(event))}
      />
      <span class="text-[11px] text-dark/65 select-none" aria-hidden="true">×</span>
      <TextInput
        type="number"
        compact
        color="neutral"
        class={edgeInputClass}
        inputSize={1}
        aria-label="Height"
        placeholder="Height"
        min={1}
        max={EXPORT_MAX_EDGE}
        step={1}
        value={form.resizeHeight === null ? '' : String(form.resizeHeight)}
        aria-invalid={error ? 'true' : undefined}
        oninput={(event: Event) => setHeight(readNumber(event))}
      />
    </div>
  </div>
{:else if form.resizeMode === 'megapixels'}
  <TextInput
    label="Megapixels"
    type="number"
    compact
    color="neutral"
    class="ring-0 focus-within:ring-1 focus-within:ring-primary"
    min={0.1}
    max={MAX_RESIZE_MEGAPIXELS}
    step={0.1}
    value={String(form.resizeMegapixels)}
    aria-invalid={error ? 'true' : undefined}
    oninput={(event: Event) => {
      form.resizeMegapixels = readNumber(event) ?? Number.NaN;
    }}
  />
{:else if form.resizeMode === 'percent'}
  <TextInput
    label="Percent"
    type="number"
    compact
    color="neutral"
    class="ring-0 focus-within:ring-1 focus-within:ring-primary"
    min={1}
    max={MAX_RESIZE_PERCENT}
    step={1}
    value={String(form.resizePercent)}
    aria-invalid={error ? 'true' : undefined}
    oninput={(event: Event) => {
      form.resizePercent = readNumber(event) ?? Number.NaN;
    }}
  />
{/if}

{#if error}
  <p class="px-1 text-[10px] leading-snug text-danger-300">{error}</p>
{/if}

{#if form.resizeMode === 'dimensions' || form.resizeMode === 'megapixels'}
  <CheckboxRow
    label="Don't enlarge"
    checked={!form.resizeEnlarge}
    onChange={(keep) => (form.resizeEnlarge = !keep)}
  />
{/if}
