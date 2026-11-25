<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown } from '@mdi/js';

  const session = trackSession;

  let open = $state(false);

  const chosen = $derived(session.cameras.find((camera) => camera.key === session.camera));
</script>

{#if session.cameras.length > 1 && chosen}
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
          aria-label="Camera: {chosen.label}"
        >
          <span class="truncate">{chosen.label} · {chosen.count}</span>
          <Icon icon={mdiChevronDown} size="16px" class="shrink-0 text-dark/65" aria-hidden />
        </Button>
      {/snippet}
      {#each session.cameras as camera (camera.key)}
        <MenuRow
          label={camera.label}
          pressed={camera.key === session.camera}
          onclick={() => {
            session.clocks.choose(camera.key);
            open = false;
          }}
        >
          <span class="shrink-0 text-caption text-dark/65 tabular-nums">
            {camera.count} · {session.driftOf(camera.key)}
          </span>
        </MenuRow>
      {/each}
    </Popover>
  </div>
{/if}
