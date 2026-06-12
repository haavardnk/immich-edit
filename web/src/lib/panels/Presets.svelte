<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiDelete, mdiPencil, mdiCheck, mdiClose, mdiContentSaveOutline } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { editsToManifest, isIdentity } from '$lib/types/edits';
  import type { Preset } from '$lib/api/presets';

  let includeGeometry = $state(false);
  let includeMasks = $state(false);
  let includeOutput = $state(false);

  let saving = $state(false);
  let newName = $state('');
  let newGroup = $state('');

  let editingId = $state<string | null>(null);
  let editName = $state('');
  let editGroup = $state('');

  $effect(() => {
    if (!presets.loaded && !presets.loading) void presets.load();
  });

  async function save(): Promise<void> {
    const name = newName.trim();
    if (!name) return;
    const manifest = editsToManifest($state.snapshot(editor.edits));
    const created = await presets.create({
      name,
      group_name: newGroup.trim() || null,
      manifest
    });
    if (created) {
      newName = '';
      newGroup = '';
      saving = false;
    }
  }

  function apply(p: Preset): void {
    void editor.applyPreset(
      p.manifest,
      { includeGeometry, includeMasks, includeOutput },
      p.name
    );
  }

  function startRename(p: Preset): void {
    editingId = p.id;
    editName = p.name;
    editGroup = p.group_name ?? '';
  }

  async function commitRename(p: Preset): Promise<void> {
    const name = editName.trim();
    if (!name) return;
    await presets.update(p.id, {
      name,
      group_name: editGroup.trim() || null,
      manifest: p.manifest
    });
    editingId = null;
  }
</script>

<div class="flex flex-col gap-2.5 pb-1">
  {#if saving}
    <div class="flex flex-col gap-1.5 rounded-lg bg-white/5 p-2.5">
      <input
        class="input bg-white/5 rounded-lg text-xs h-auto py-1.5 min-h-0"
        placeholder="Preset name"
        bind:value={newName}
      />
      <input
        class="input bg-white/5 rounded-lg text-xs h-auto py-1.5 min-h-0"
        placeholder="Group (optional)"
        bind:value={newGroup}
      />
      <div class="flex gap-1.5">
        <button
          class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-immich-dark-primary/80 hover:bg-immich-dark-primary text-xs transition-colors disabled:opacity-40"
          disabled={!newName.trim()}
          onclick={() => void save()}
        >
          <Icon path={mdiCheck} size={14} />
          Save
        </button>
        <button
          class="flex items-center justify-center py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
          onclick={() => (saving = false)}
          aria-label="Cancel"
        >
          <Icon path={mdiClose} size={14} />
        </button>
      </div>
    </div>
  {:else}
    <button
      class="flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      disabled={!editor.assetId || isIdentity(editor.edits)}
      onclick={() => {
        saving = true;
      }}
      title={isIdentity(editor.edits) ? 'No edits to save' : 'Save current edits as preset'}
    >
      <Icon path={mdiContentSaveOutline} size={14} />
      Save current as preset
    </button>
  {/if}

  <div class="flex flex-col gap-1 text-[11px] text-immich-dark-fg/60">
    <span class="uppercase tracking-wider text-[10px] text-immich-dark-fg/40">Apply includes</span>
    <label class="flex items-center gap-2">
      <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeGeometry} />
      Geometry &amp; crop
    </label>
    <label class="flex items-center gap-2">
      <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeMasks} />
      Masks
    </label>
    <label class="flex items-center gap-2">
      <input type="checkbox" class="checkbox checkbox-xs" bind:checked={includeOutput} />
      Tonemap
    </label>
  </div>

  {#if presets.presets.length === 0}
    <div class="text-xs text-immich-dark-fg/30 py-1">No presets yet.</div>
  {:else}
    {#each presets.grouped as { group, items } (group)}
      {#if group}
        <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40 pt-1">{group}</div>
      {/if}
      <div class="flex flex-col gap-1">
        {#each items as p (p.id)}
          {#if editingId === p.id}
            <div class="flex flex-col gap-1.5 rounded-lg bg-white/5 p-2">
              <input
                class="input bg-white/5 rounded-lg text-xs h-auto py-1 min-h-0"
                placeholder="Name"
                bind:value={editName}
              />
              <input
                class="input bg-white/5 rounded-lg text-xs h-auto py-1 min-h-0"
                placeholder="Group (optional)"
                bind:value={editGroup}
              />
              <div class="flex gap-1.5">
                <button
                  class="flex-1 flex items-center justify-center gap-1 py-1 rounded-lg bg-immich-dark-primary/80 hover:bg-immich-dark-primary text-xs transition-colors disabled:opacity-40"
                  disabled={!editName.trim()}
                  onclick={() => void commitRename(p)}
                >
                  <Icon path={mdiCheck} size={13} />
                </button>
                <button
                  class="flex items-center justify-center py-1 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
                  onclick={() => (editingId = null)}
                  aria-label="Cancel"
                >
                  <Icon path={mdiClose} size={13} />
                </button>
              </div>
            </div>
          {:else}
            <div class="flex items-center gap-1 group">
              <button
                class="flex-1 text-left px-2 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors truncate disabled:opacity-40 disabled:cursor-not-allowed"
                disabled={!editor.assetId || editor.saving}
                onclick={() => apply(p)}
                title="Apply {p.name}"
              >
                {p.name}
              </button>
              <button
                class="flex items-center justify-center p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-immich-dark-fg hover:bg-white/10 transition-colors"
                onclick={() => startRename(p)}
                aria-label="Rename {p.name}"
                title="Rename"
              >
                <Icon path={mdiPencil} size={13} />
              </button>
              <button
                class="flex items-center justify-center p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-immich-dark-error hover:bg-white/10 transition-colors"
                onclick={() => void presets.remove(p.id)}
                aria-label="Delete {p.name}"
                title="Delete"
              >
                <Icon path={mdiDelete} size={13} />
              </button>
            </div>
          {/if}
        {/each}
      </div>
    {/each}
  {/if}
</div>
