<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import PresetIncludeToggles from './preset/IncludeToggles.svelte';
  import PresetPicker from './preset/PresetPicker.svelte';
  import {
    mdiDelete,
    mdiPencil,
    mdiCheck,
    mdiClose,
    mdiContentSaveOutline,
    mdiAutoFix
  } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { editsToManifest, isIdentity } from '$lib/types/edits';

  let includeGeometry = $state(false);
  let includeMasks = $state(false);

  let saving = $state(false);
  let newName = $state('');
  let newGroup = $state('');

  let selectedId = $state<string | null>(null);
  let editing = $state(false);
  let editName = $state('');
  let editGroup = $state('');

  const selected = $derived(presets.presets.find((p) => p.id === selectedId) ?? null);

  $effect(() => {
    if (!presets.loaded && !presets.loading) void presets.load();
  });

  $effect(() => {
    if (selectedId && !presets.presets.some((p) => p.id === selectedId)) {
      selectedId = null;
      editing = false;
    }
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
      selectedId = created.id;
    }
  }

  function apply(): void {
    if (!selected) return;
    void editor.applyPreset(selected.manifest, { includeGeometry, includeMasks }, selected.name);
  }

  function startRename(): void {
    if (!selected) return;
    editing = true;
    editName = selected.name;
    editGroup = selected.group_name ?? '';
  }

  async function commitRename(): Promise<void> {
    if (!selected) return;
    const name = editName.trim();
    if (!name) return;
    await presets.update(selected.id, {
      name,
      group_name: editGroup.trim() || null,
      manifest: selected.manifest
    });
    editing = false;
  }

  async function remove(): Promise<void> {
    if (!selected) return;
    await presets.remove(selected.id);
    selectedId = null;
    editing = false;
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

  {#if presets.presets.length === 0}
    <div class="text-xs text-immich-dark-fg/30 py-1">No presets yet.</div>
  {:else}
    <div class="flex items-center gap-1">
      <div class="flex-1 min-w-0">
        <PresetPicker bind:selectedId />
      </div>
      <button
        class="flex items-center justify-center p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-immich-dark-fg hover:bg-white/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        disabled={!selected}
        onclick={startRename}
        aria-label="Rename preset"
        title="Rename"
      >
        <Icon path={mdiPencil} size={13} />
      </button>
      <button
        class="flex items-center justify-center p-1.5 rounded-lg text-immich-dark-fg/40 hover:text-immich-dark-error hover:bg-white/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
        disabled={!selected}
        onclick={() => void remove()}
        aria-label="Delete preset"
        title="Delete"
      >
        <Icon path={mdiDelete} size={13} />
      </button>
    </div>

    {#if editing && selected}
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
            onclick={() => void commitRename()}
          >
            <Icon path={mdiCheck} size={13} />
          </button>
          <button
            class="flex items-center justify-center py-1 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors"
            onclick={() => (editing = false)}
            aria-label="Cancel"
          >
            <Icon path={mdiClose} size={13} />
          </button>
        </div>
      </div>
    {/if}

    <PresetIncludeToggles bind:includeGeometry bind:includeMasks />

    <button
      class="flex items-center justify-center gap-2 py-2 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 text-xs font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      disabled={!selected || !editor.assetId || editor.saving}
      onclick={apply}
      title={selected ? `Apply ${selected.name}` : 'Select a preset'}
    >
      <Icon path={mdiAutoFix} size={14} />
      {selected ? `Apply ${selected.name}` : 'Apply preset'}
    </button>
  {/if}
</div>
