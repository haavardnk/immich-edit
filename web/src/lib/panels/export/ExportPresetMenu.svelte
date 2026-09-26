<script lang="ts">
  import Popover from '$lib/components/Popover.svelte';
  import { Icon } from '@immich/ui';
  import { mdiChevronDown, mdiContentSaveOutline, mdiDeleteOutline } from '@mdi/js';
  import { exportPresets } from './exportPresets.svelte';

  let open = $state(false);
  let name = $state('');

  function onOpenChange(next: boolean): void {
    open = next;
    if (next && !exportPresets.loaded) void exportPresets.load();
  }

  async function save(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    await exportPresets.save(trimmed);
    name = '';
  }
</script>

<Popover {open} {onOpenChange} align="start" contentClass="w-64">
  {#snippet trigger(props)}
    <button
      {...props}
      type="button"
      class="flex items-center gap-1 self-start rounded px-1.5 py-0.5 text-[11px] text-dark/75 hover:bg-white/5 hover:text-dark"
    >
      Export presets
      <Icon icon={mdiChevronDown} size="14" />
    </button>
  {/snippet}
  <ul class="py-1" aria-label="Export presets">
    {#each exportPresets.presets as preset (preset.id)}
      <li class="group flex items-center hover:bg-white/5">
        <button
          type="button"
          class="min-w-0 flex-1 truncate px-3 py-1.5 text-left text-xs"
          onclick={() => {
            exportPresets.apply(preset);
            open = false;
          }}
        >
          {preset.name}
        </button>
        <button
          type="button"
          class="px-2 py-1.5 text-dark/50 hover:text-red-400"
          aria-label="Delete export preset {preset.name}"
          title="Delete"
          onclick={() => void exportPresets.remove(preset)}
        >
          <Icon icon={mdiDeleteOutline} size="14" />
        </button>
      </li>
    {:else}
      <li class="px-3 py-1.5 text-xs text-dark/50">
        {exportPresets.loaded ? 'No saved presets' : 'Loading…'}
      </li>
    {/each}
  </ul>
  <form class="flex items-center gap-1 border-t border-dark/10 p-2" onsubmit={save}>
    <input
      bind:value={name}
      maxlength={80}
      placeholder="Save current as…"
      aria-label="Export preset name"
      class="min-w-0 flex-1 rounded border border-dark/15 bg-transparent px-2 py-1 text-xs"
    />
    <button
      type="submit"
      class="p-1 text-dark/65 hover:text-dark disabled:opacity-40"
      aria-label="Save export preset"
      title="Save (overwrites a preset with the same name)"
      disabled={!name.trim()}
    >
      <Icon icon={mdiContentSaveOutline} size="16" />
    </button>
  </form>
</Popover>
