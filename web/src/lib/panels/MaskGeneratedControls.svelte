<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent } from '$lib/types/edits';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const meta = $derived(component.generated);

  let syncedId = $state<string | null>(null);
  let grow = $state(0);
  let featherPx = $state(0);
  let depthMin = $state(0);
  let depthMax = $state(1);
  let depthSoftness = $state(0.1);

  $effect(() => {
    if (component.id === syncedId) return;
    syncedId = component.id;
    grow = meta?.grow ?? 0;
    featherPx = meta?.feather ?? 0;
    depthMin = meta?.range?.min ?? 0;
    depthMax = meta?.range?.max ?? 1;
    depthSoftness = meta?.range?.softness ?? 0.1;
  });

  function label(kind: string): string {
    if (kind === 'subject') return 'Subject';
    if (kind === 'people') return 'People';
    if (kind === 'sky') return 'Sky';
    if (kind === 'depth') return 'Depth';
    if (kind === 'semantic') return 'Scene';
    if (kind === 'click') return 'Click to select';
    return kind;
  }

  async function rebake(): Promise<void> {
    if (!meta) return;
    const range =
      meta.kind === 'depth'
        ? { min: depthMin, max: depthMax, softness: depthSoftness }
        : undefined;
    await editor.rebakeGeneratedComponent(layerId, component.id, grow, featherPx, range);
  }
</script>

{#if meta}
  <div class="mt-2 flex flex-col gap-2.5">
    <div class="px-1 text-[10px] uppercase tracking-wider text-immich-dark-fg/40">
      {label(meta.kind)}{meta.class ? ` · ${meta.class}` : ''} · {meta.model_id}
    </div>
    {#if meta.painted}
      <div class="mx-1 rounded bg-amber-500/10 px-2 py-1.5 text-[11px] text-amber-200/90">
        You have painted on this mask. Moving these sliders regenerates it from the model and
        discards those strokes.
      </div>
    {/if}
    {#if meta.kind === 'depth'}
      <SliderRow
        label="Near"
        value={depthMin}
        min={0}
        max={1}
        step={0.01}
        defaultValue={0}
        onLive={(v: number) => (depthMin = Math.min(v, depthMax))}
        onCommit={rebake}
        format={(v: number) => v.toFixed(2)}
      />
      <SliderRow
        label="Far"
        value={depthMax}
        min={0}
        max={1}
        step={0.01}
        defaultValue={1}
        onLive={(v: number) => (depthMax = Math.max(v, depthMin))}
        onCommit={rebake}
        format={(v: number) => v.toFixed(2)}
      />
      <SliderRow
        label="Softness"
        value={depthSoftness}
        min={0}
        max={1}
        step={0.01}
        defaultValue={0.1}
        onLive={(v: number) => (depthSoftness = v)}
        onCommit={rebake}
        format={(v: number) => v.toFixed(2)}
      />
    {/if}
    <SliderRow
      label="Grow"
      value={grow}
      min={-32}
      max={32}
      step={1}
      defaultValue={0}
      onLive={(v: number) => (grow = v)}
      onCommit={rebake}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Feather"
      value={featherPx}
      min={0}
      max={32}
      step={1}
      defaultValue={0}
      onLive={(v: number) => (featherPx = v)}
      onCommit={rebake}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
{/if}
