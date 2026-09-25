<script lang="ts">
  import DeleteConfirmation from '$lib/components/DeleteConfirmation.svelte';
  import RangeSlider from '$lib/components/editor/controls/RangeSlider.svelte';
  import { ApiError } from '$lib/api/client';
  import {
    deleteWatermark,
    importWatermark,
    listWatermarks,
    type WatermarkMeta
  } from '$lib/api/watermarks';
  import { session } from '$lib/stores/session.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { Button, Field, Select } from '@immich/ui';
  import { mdiUpload } from '@mdi/js';
  import { WATERMARK_ANCHORS, WATERMARK_PERCENT } from './watermark';
  import type { ExportForm } from './settings';

  let { form = $bindable<ExportForm>() }: { form: ExportForm } = $props();

  let watermarks = $state<WatermarkMeta[]>([]);
  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state(false);
  let fileInput: HTMLInputElement | null = $state(null);

  const options = $derived([
    { value: 'none', label: 'None' },
    ...watermarks.map((w) => ({ value: w.id, label: w.name }))
  ]);
  const selected = $derived(watermarks.find((w) => w.id === form.watermarkId) ?? null);
  const sliders = $derived([
    {
      label: 'Size',
      value: form.watermarkSize,
      range: WATERMARK_PERCENT.size,
      set: (v: number) => (form.watermarkSize = v)
    },
    {
      label: 'Opacity',
      value: form.watermarkOpacity,
      range: WATERMARK_PERCENT.opacity,
      set: (v: number) => (form.watermarkOpacity = v)
    },
    {
      label: 'Inset',
      value: form.watermarkInset,
      range: WATERMARK_PERCENT.inset,
      set: (v: number) => (form.watermarkInset = v)
    }
  ]);

  $effect(() => {
    if (!loaded) void load();
  });

  async function load(): Promise<void> {
    try {
      watermarks = await listWatermarks();
      if (form.watermarkId && !watermarks.some((w) => w.id === form.watermarkId)) {
        form.watermarkId = null;
      }
    } catch (e) {
      watermarks = [];
      toasts.fail('watermarks', e);
    }
    loaded = true;
  }

  function select(id: string | null): void {
    form.watermarkId = id;
    pendingDelete = false;
  }

  async function onFile(e: Event): Promise<void> {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    const name = file.name.replace(/\.[^.]+$/, '');
    const bytes = new Uint8Array(await file.arrayBuffer());
    importing = true;
    try {
      const meta = await importWatermark(name, bytes);
      await load();
      select(meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        if (existing && watermarks.some((w) => w.id === existing)) {
          select(existing);
          toasts.push('info', 'Watermark already imported — selected existing.');
        }
      } else if (err instanceof ApiError && err.status === 400) {
        toasts.push('error', `Invalid watermark: ${err.message}`);
      } else {
        toasts.push('error', 'Watermark import failed.');
      }
    } finally {
      importing = false;
    }
  }

  async function confirmDelete(): Promise<void> {
    if (!selected) return;
    try {
      await deleteWatermark(selected.id);
      select(null);
      await load();
    } catch {
      toasts.push('error', 'Watermark delete failed.');
    }
  }
</script>

<input
  bind:this={fileInput}
  type="file"
  accept="image/png"
  class="hidden"
  onchange={(e) => void onFile(e)}
/>

<div class="flex items-end gap-1">
  <div class="min-w-0 flex-1">
    <Field label="Watermark" size="tiny">
      <Select
        size="tiny"
        class="editor-compact-select editor-compact-field"
        {options}
        value={form.watermarkId ?? 'none'}
        onChange={(id) => select(id === 'none' ? null : id)}
      />
    </Field>
  </div>
  {#if selected && session.isAdmin}
    <DeleteConfirmation
      bind:pending={pendingDelete}
      label="Delete watermark"
      confirmLabel="Confirm delete watermark"
      onconfirm={confirmDelete}
    />
  {/if}
</div>

{#if session.isAdmin}
  <Button
    type="button"
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-7 panel-action"
    leadingIcon={mdiUpload}
    disabled={importing}
    onclick={() => fileInput?.click()}
  >
    {importing ? 'Importing…' : 'Import PNG watermark'}
  </Button>
{/if}

{#if selected}
  {#each sliders as slider (slider.label)}
    <div class="panel-row h-7 items-center">
      <span class="editor-compact-label select-none">{slider.label}</span>
      <RangeSlider
        label={`Watermark ${slider.label.toLowerCase()}`}
        min={slider.range.min}
        max={slider.range.max}
        step={1}
        value={slider.value}
        oninput={(event) => slider.set((event.currentTarget as HTMLInputElement).valueAsNumber)}
      />
      <span class="px-1 text-right font-mono text-[10px] tabular-nums text-dark/65"
        >{slider.value}%</span
      >
    </div>
  {/each}
  <div class="panel-row items-center py-0.5">
    <span class="editor-compact-label select-none">Position</span>
    <div role="group" aria-label="Watermark position" class="grid w-fit grid-cols-3 gap-1">
      {#each WATERMARK_ANCHORS as anchor (anchor.value)}
        <button
          type="button"
          class="size-5 rounded-sm transition-colors focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary {form.watermarkAnchor ===
          anchor.value
            ? 'bg-primary'
            : 'bg-neutral-700 hover:bg-neutral-600'}"
          aria-label={anchor.label}
          title={anchor.label}
          aria-pressed={form.watermarkAnchor === anchor.value}
          onclick={() => (form.watermarkAnchor = anchor.value)}
        ></button>
      {/each}
    </div>
  </div>
{/if}
