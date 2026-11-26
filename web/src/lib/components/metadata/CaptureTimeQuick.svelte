<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import WallTimeField from '#lib/components/metadata/WallTimeField.svelte';
  import ZoneSelect from '#lib/components/metadata/ZoneSelect.svelte';
  import { clockPhoto } from '#lib/metadata/captureTimes.ts';
  import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';
  import type { AssetSummary } from '#lib/types/album.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiPencilOutline } from '@mdi/js';

  let { asset, value }: { asset: AssetSummary; value: string } = $props();

  let editing = $state.raw<AssetSummary | null>(null);
  let wall = $state(0);
  let zone = $state('');
  let saving = $state(false);

  const open = $derived(editing !== null && editing.id === asset.id);

  function setOpen(next: boolean): void {
    if (!next) {
      editing = null;
      return;
    }
    const photo = clockPhoto(asset);
    wall = photo?.wall ?? Date.now();
    zone = photo?.zone ?? captureEditor.fallback;
    editing = asset;
  }

  async function save(): Promise<void> {
    const target = editing;
    if (!target) return;
    saving = true;
    try {
      if (await captureEditor.saveOne(target, wall, zone)) editing = null;
    } finally {
      saving = false;
    }
  }

  function more(): void {
    const target = editing;
    editing = null;
    if (target) captureEditor.open([target]);
  }
</script>

<Popover
  {open}
  onOpenChange={setOpen}
  appearance="panel"
  align="end"
  title="Capture time"
  subtitle={asset.originalFileName}
  contentClass="w-84 p-3"
  tooltip="Change the capture time"
>
  {#snippet trigger(props)}
    <Button
      {...props}
      size="tiny"
      variant="ghost"
      color="secondary"
      class="-me-1.5 h-6 min-w-0 gap-1.5 px-1.5 text-xs font-medium text-light-950/90 tabular-nums touch:h-9 touch:text-body"
      title=""
      aria-label="Taken {value}"
    >
      <span class="min-w-0 truncate">{value}</span>
      <Icon icon={mdiPencilOutline} size="12px" class="shrink-0 text-dark/65" aria-hidden="true" />
    </Button>
  {/snippet}
  <div class="flex flex-col gap-3">
    <WallTimeField label="Date and time" value={wall} onChange={(next) => (wall = next)} />
    <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
      <span class="min-w-0 flex-1 text-sm text-light-950">Time zone</span>
      <ZoneSelect
        label="Time zone"
        value={zone}
        at={wall}
        class="w-56"
        onPick={(next) => (zone = next)}
      />
    </div>
    <div class="flex items-center gap-2">
      <Button size="small" variant="ghost" color="primary" class="me-auto px-1" onclick={more}>
        More…
      </Button>
      <Button size="small" variant="ghost" color="secondary" onclick={() => setOpen(false)}>
        Cancel
      </Button>
      <Button size="small" loading={saving} onclick={save}>Save</Button>
    </div>
  </div>
</Popover>
