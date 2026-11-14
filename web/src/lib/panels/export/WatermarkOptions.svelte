<script lang="ts">
  import DeleteConfirmation from '#lib/components/DeleteConfirmation.svelte';
  import RangeSlider from '#lib/components/editor/controls/RangeSlider.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { ApiError } from '#lib/api/client.ts';
  import { deleteWatermark, importWatermark, watermarkPngUrl } from '#lib/api/watermarks.ts';
  import { watermarkLibrary } from '#lib/panels/export/watermarkLibrary.svelte.ts';
  import { watermarkPreview } from '#lib/panels/export/watermarkPreview.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import { Button, Icon, IconButton, Tooltip } from '@immich/ui';
  import { mergeProps, RadioGroup } from 'bits-ui';
  import { mdiBlockHelper, mdiClose, mdiLoading, mdiUpload } from '@mdi/js';
  import { WATERMARK_ANCHORS, WATERMARK_PERCENT } from './watermark';
  import type { ExportFormValues } from './settings';

  let {
    form = $bindable<ExportFormValues>(),
    onpick
  }: { form: ExportFormValues; onpick?: () => void } = $props();

  let loaded = $state(false);
  let importing = $state(false);
  let pendingDelete = $state(false);
  let importError = $state<{ file: string; message: string } | null>(null);
  let fileInput: HTMLInputElement | null = $state(null);

  const NONE = 'none';
  const tiles = $derived(
    [...watermarkLibrary.items].sort((a, b) => b.created_at.localeCompare(a.created_at))
  );
  const tile =
    'flex w-16 shrink-0 flex-col items-center gap-1 rounded-lg p-1 text-[10px] text-dark/65 outline-none focus-visible:ring-2 focus-visible:ring-primary touch:w-20 touch:text-xs';
  const face =
    'flex h-11 w-full items-center justify-center overflow-hidden rounded-md bg-light-200 touch:h-14';
  const selected = $derived(watermarkLibrary.items.find((w) => w.id === form.watermarkId) ?? null);
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

  $effect(() => watermarkPreview.hold());

  async function load(): Promise<void> {
    const listed = await watermarkLibrary.load();
    if (listed && form.watermarkId && !watermarkLibrary.has(form.watermarkId)) {
      form.watermarkId = null;
    }
    loaded = true;
  }

  function select(id: string | null): void {
    form.watermarkId = id;
    pendingDelete = false;
  }

  function pick(id: string | null): void {
    select(id);
    if (id) onpick?.();
  }

  async function onFile(e: Event): Promise<void> {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    const name = file.name.replace(/\.[^.]+$/, '');
    const bytes = new Uint8Array(await file.arrayBuffer());
    importing = true;
    importError = null;
    try {
      const meta = await importWatermark(name, bytes);
      await load();
      select(meta.id);
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        const existing = err.message.split(':').pop()?.trim();
        await load();
        if (existing && watermarkLibrary.has(existing)) {
          select(existing);
          toasts.push('info', 'Watermark already imported — selected existing.');
        }
      } else if (err instanceof ApiError && err.status === 400) {
        importError = { file: file.name, message: `Invalid watermark: ${err.message}` };
      } else {
        importError = { file: file.name, message: 'Watermark import failed.' };
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

<div class="flex items-start gap-1 overflow-x-auto pb-1 scrollbar-hidden">
  {#if session.isAdmin}
    <Button
      size="tiny"
      variant="ghost"
      color="primary"
      class="{tile} h-auto text-primary"
      disabled={importing}
      aria-label="Import PNG watermark"
      onclick={() => fileInput?.click()}
    >
      <span class="{face} border border-primary/40 bg-primary/10">
        <Icon
          icon={importing ? mdiLoading : mdiUpload}
          size="18px"
          class={importing ? 'animate-spin' : ''}
          aria-hidden="true"
        />
      </span>
      <span class="w-full truncate text-center">Import</span>
    </Button>
  {/if}
  <RadioGroup.Root
    value={form.watermarkId ?? NONE}
    onValueChange={(id) => pick(id === NONE ? null : id)}
    orientation="horizontal"
    aria-label="Watermark"
    class="flex items-start gap-1"
  >
    <RadioGroup.Item value={NONE} class="{tile} group aria-checked:text-dark">
      <span
        class="{face} group-aria-checked:ring-2 group-aria-checked:ring-primary group-aria-checked:ring-inset"
      >
        <Icon icon={mdiBlockHelper} size="18px" aria-hidden="true" />
      </span>
      <span class="w-full truncate text-center">None</span>
    </RadioGroup.Item>
    {#each tiles as mark (mark.id)}
      <TruncationTooltip text={mark.name}>
        {#snippet child({ props })}
          <RadioGroup.Item {...props} value={mark.id} class="{tile} group aria-checked:text-dark">
            <span
              class="{face} group-aria-checked:ring-2 group-aria-checked:ring-primary group-aria-checked:ring-inset"
            >
              <img
                src={watermarkPngUrl(mark.id)}
                alt=""
                class="max-h-full max-w-full object-contain p-1"
              />
            </span>
            <span data-truncate class="w-full truncate text-center">{mark.name}</span>
          </RadioGroup.Item>
        {/snippet}
      </TruncationTooltip>
    {/each}
  </RadioGroup.Root>
</div>

{#if importError}
  <div
    role="alert"
    class="flex items-start gap-2 rounded-lg border border-danger/30 bg-danger/10 px-2.5 py-2 text-[11px] leading-snug text-danger-800 touch:text-sm"
  >
    <div class="min-w-0 flex-1">
      <p class="font-mono break-all">{importError.file}</p>
      <p>{importError.message}</p>
    </div>
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiClose}
      title="Dismiss"
      aria-label="Dismiss watermark error"
      class="shrink-0 touch:size-9"
      onclick={() => (importError = null)}
    />
  </div>
{/if}

{#if selected && session.isAdmin}
  <div class="flex justify-end">
    <DeleteConfirmation
      bind:pending={pendingDelete}
      label="Delete watermark"
      confirmLabel="Confirm delete watermark"
      onconfirm={confirmDelete}
    />
  </div>
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
        <Tooltip text={anchor.label}>
          {#snippet child({ props })}
            <button
              {...mergeProps(props, {
                type: 'button' as const,
                class: `size-5 rounded-sm transition-colors focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary ${
                  form.watermarkAnchor === anchor.value
                    ? 'bg-primary'
                    : 'bg-light-300 hover:bg-light-400'
                }`,
                'aria-label': anchor.label,
                'aria-pressed': form.watermarkAnchor === anchor.value,
                onclick: () => (form.watermarkAnchor = anchor.value)
              })}
            ></button>
          {/snippet}
        </Tooltip>
      {/each}
    </div>
  </div>
{/if}
