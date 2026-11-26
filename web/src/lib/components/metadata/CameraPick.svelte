<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import type { AllCameras, Camera } from '#lib/metadata/cameras.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';

  let {
    cameras,
    chosen,
    all,
    detail,
    onChoose
  }: {
    cameras: readonly Camera[];
    chosen: string;
    all?: AllCameras;
    detail: (key: string) => string;
    onChoose: (key: string) => void;
  } = $props();

  let open = $state(false);

  const current = $derived(
    all && chosen === all.key
      ? { label: 'All cameras', count: all.count }
      : cameras.find((camera) => camera.key === chosen)
  );

  function choose(key: string): void {
    onChoose(key);
    open = false;
  }
</script>

{#if cameras.length > 1 && current}
  <div class="flex items-center gap-3">
    <span class="min-w-0 flex-1 text-sm text-light-950">Camera</span>
    <Popover {open} onOpenChange={(next) => (open = next)} align="end" title="Camera">
      {#snippet trigger(props)}
        <Button
          {...props}
          size="small"
          variant="ghost"
          color="secondary"
          class="h-8 max-w-56 gap-1 bg-light-100 ps-2.5 pe-1.5 ring-1 ring-hairline touch:h-11"
          aria-label="Camera: {current.label}"
        >
          <span class="truncate">{current.label} · {current.count}</span>
          <Icon icon={mdiChevronDown} size="16px" class="shrink-0 text-dark/65" aria-hidden />
        </Button>
      {/snippet}
      {#if all}
        <MenuRow label="All cameras" pressed={chosen === all.key} onclick={() => choose(all.key)}>
          <span class="shrink-0 text-caption text-dark/65 tabular-nums">{all.count}</span>
        </MenuRow>
        <div class="my-1 h-px bg-hairline" role="separator"></div>
      {/if}
      {#each cameras as camera (camera.key)}
        <MenuRow
          label={camera.label}
          pressed={camera.key === chosen}
          onclick={() => choose(camera.key)}
        >
          <span class="shrink-0 text-caption text-dark/65 tabular-nums">
            {camera.count} · {detail(camera.key)}
          </span>
        </MenuRow>
      {/each}
    </Popover>
  </div>
{/if}
