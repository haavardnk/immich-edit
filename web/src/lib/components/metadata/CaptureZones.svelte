<script lang="ts">
  import ZoneSelect from '#lib/components/metadata/ZoneSelect.svelte';
  import { asShotText, clockExample } from '#lib/metadata/captureLabels.ts';
  import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';

  const AS_SHOT = 'as-shot';
  const SAME_ZONE = 'same-zone';
  const MIXED = 'mixed';

  const editor = captureEditor;
  const once = $derived(editor.mode === 'once');
  const mixed = $derived(editor.zone === null);
  const sample = $derived(editor.scoped[0] ?? null);
  const at = $derived(sample?.wall ?? Date.now());
  const asShot = $derived(
    asShotText(
      editor.scoped.map((photo) => ({
        zone: photo.zone ?? editor.fallback,
        at: photo.stored ?? photo.wall
      }))
    )
  );
  const choices = $derived([
    { id: AS_SHOT, label: asShot },
    ...(mixed ? [{ id: MIXED, label: 'Mixed' }] : [])
  ]);
  const example = $derived(
    !once && sample && editor.takenIn && editor.clockZone
      ? clockExample(sample.wall, editor.shownOf(sample), editor.takenIn)
      : null
  );
</script>

<div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
  <span class="min-w-0 flex-1 text-sm text-light-950">Time zone</span>
  {#if once}
    <ZoneSelect
      label="Time zone"
      value={mixed ? MIXED : (editor.takenIn ?? editor.fallback)}
      {at}
      choices={mixed ? [{ id: MIXED, label: 'Mixed' }] : []}
      onPick={(zone) => {
        if (zone !== MIXED) editor.setTakenIn(zone);
      }}
    />
  {:else}
    <ZoneSelect
      label="Time zone"
      value={mixed ? MIXED : (editor.takenIn ?? AS_SHOT)}
      {at}
      {choices}
      onPick={(id) => {
        if (id !== MIXED) editor.setTakenIn(id === AS_SHOT ? null : id);
      }}
    />
  {/if}
</div>
{#if !once && editor.takenIn}
  <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
    <span class="min-w-0 flex-1 text-sm text-light-950">Camera time zone</span>
    <ZoneSelect
      label="Camera time zone"
      value={editor.clockZone ?? SAME_ZONE}
      {at}
      choices={[{ id: SAME_ZONE, label: 'Same zone' }]}
      onPick={(id) => editor.setClockZone(id === SAME_ZONE ? null : id)}
    />
  </div>
  {#if example}
    <p class="-mt-1 text-end text-xs text-dark/65 tabular-nums touch:text-start touch:text-sm">
      {example}
    </p>
  {/if}
{/if}
