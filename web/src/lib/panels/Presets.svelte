<script lang="ts">
  import PresetIncludeToggles from './preset/IncludeToggles.svelte';
  import PresetPicker from './preset/PresetPicker.svelte';
  import DeleteConfirmation from '$lib/components/DeleteConfirmation.svelte';
  import EditableLabel from '$lib/components/EditableLabel.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { mdiPencil, mdiCheck, mdiClose, mdiContentSaveOutline, mdiAutoFix } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { editsToManifest, isIdentity } from '$lib/types/edits';
  import { Button, IconButton } from '@immich/ui';

  let includeGeometry = $state(false);
  let includeMasks = $state(false);

  let saving = $state(false);
  let newName = $state('');
  let newGroup = $state('');

  let selectedId = $state<string | null>(null);
  let editing = $state(false);
  let editName = $state('');
  let editGroup = $state('');
  let pendingDelete = $state(false);

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

  $effect(() => {
    const _tracked = selectedId;
    pendingDelete = false;
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

<div class="flex flex-col gap-1 pb-1">
  {#if saving}
    <div class="flex flex-col gap-1">
      <TextInput compact placeholder="Preset name" aria-label="Preset name" bind:value={newName} />
      <TextInput
        compact
        placeholder="Group (optional)"
        aria-label="Preset group"
        bind:value={newGroup}
      />
      <div class="flex gap-1">
        <Button
          size="tiny"
          color="primary"
          class="h-7 flex-1"
          leadingIcon={mdiCheck}
          disabled={!newName.trim()}
          onclick={() => void save()}
        >
          Save
        </Button>
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          class="size-7 panel-action"
          icon={mdiClose}
          onclick={() => (saving = false)}
          aria-label="Cancel"
        />
      </div>
    </div>
  {:else}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-7 panel-action"
      leadingIcon={mdiContentSaveOutline}
      title={isIdentity(editor.edits) ? 'No edits to save' : 'Save current edits as preset'}
      disabled={!editor.assetId || isIdentity(editor.edits)}
      onclick={() => {
        saving = true;
      }}
    >
      Save current as preset
    </Button>
  {/if}

  {#if presets.presets.length === 0}
    <div class="px-1 py-1 text-xs text-dark/65">No presets yet.</div>
  {:else}
    <div class="flex items-center gap-1">
      <div class="flex-1 min-w-0">
        <PresetPicker bind:selectedId />
      </div>
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiPencil}
        title="Rename"
        aria-label="Rename preset"
        disabled={!selected}
        onclick={startRename}
      />
      <DeleteConfirmation
        bind:pending={pendingDelete}
        label="Delete preset"
        title="Delete"
        confirmLabel="Confirm delete preset"
        disabled={!selected}
        onconfirm={remove}
      />
    </div>

    {#if editing && selected}
      <div class="flex flex-col gap-1">
        <EditableLabel
          compact
          placeholder="Name"
          ariaLabel="Preset name"
          bind:value={editName}
          oncommit={commitRename}
          oncancel={() => (editing = false)}
        />
        <TextInput
          compact
          placeholder="Group (optional)"
          aria-label="Preset group"
          bind:value={editGroup}
        />
        <div class="flex gap-1">
          <Button
            size="tiny"
            color="primary"
            class="h-7 flex-1"
            leadingIcon={mdiCheck}
            disabled={!editName.trim()}
            onclick={() => void commitRename()}
          >
            Rename
          </Button>
          <IconButton
            size="tiny"
            variant="ghost"
            color="secondary"
            class="size-7 panel-action"
            icon={mdiClose}
            onclick={() => (editing = false)}
            aria-label="Cancel"
          />
        </div>
      </div>
    {/if}

    <PresetIncludeToggles bind:includeGeometry bind:includeMasks />

    <Button
      size="tiny"
      color="primary"
      leadingIcon={mdiAutoFix}
      title={selected ? `Apply ${selected.name}` : 'Select a preset'}
      disabled={!selected || !editor.assetId || editor.saving}
      onclick={apply}
    >
      {selected ? `Apply ${selected.name}` : 'Apply preset'}
    </Button>
  {/if}
</div>
