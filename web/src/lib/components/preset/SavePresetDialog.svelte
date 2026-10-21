<script lang="ts">
  import { untrack } from 'svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import SearchableSelect from '$lib/components/SearchableSelect.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import type { Preset } from '$lib/api/presets';
  import { editsToManifest } from '$lib/edits/manifest';
  import { editor } from '$lib/stores/editor.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { Button } from '@immich/ui';

  let { preset, onClose }: { preset?: Preset; onClose: () => void } = $props();

  const initial = untrack(() => preset);
  let name = $state(initial?.name ?? '');
  let group = $state<string[]>(initial?.group_name ? [initial.group_name] : []);
  let newGroups = $state<string[]>([]);
  let saving = $state(false);
  let nameInput = $state<HTMLInputElement | null>(null);

  $effect(() => nameInput?.focus());

  const groups = $derived([
    ...new Set([...presets.grouped.map((g) => g.group).filter(Boolean), ...newGroups])
  ]);

  function addGroup(value: string): void {
    newGroups.push(value);
    group = [value];
  }

  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed || saving) return;
    saving = true;
    const input = {
      name: trimmed,
      group_name: group[0] ?? null,
      manifest: preset?.manifest ?? editsToManifest($state.snapshot(editor.edits))
    };
    const saved = preset ? await presets.update(preset.id, input) : await presets.create(input);
    saving = false;
    if (saved) onClose();
  }
</script>

<Dialog title={preset ? 'Edit preset' : 'Save preset'} {onClose}>
  <form class="flex flex-col gap-4" onsubmit={(e) => void save(e)}>
    <div class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-4 gap-y-2">
      <span class="editor-compact-label select-none" aria-hidden="true">Name</span>
      <TextInput
        compact
        color="neutral"
        class="ring-0 focus-within:ring-1 focus-within:ring-primary [&_input]:h-7"
        aria-label="Preset name"
        bind:ref={nameInput}
        bind:value={name}
      />
      <span class="editor-compact-label select-none" aria-hidden="true">Group</span>
      <SearchableSelect
        compact
        color="neutral"
        options={groups}
        bind:selected={group}
        commitOnClose
        getId={(g) => g}
        getLabel={(g) => g}
        multiple={false}
        placeholder="Choose or create…"
        label="Preset group"
        onCreate={addGroup}
      />
    </div>
    <div class="flex justify-end gap-2">
      <Button type="button" size="small" variant="ghost" color="secondary" onclick={onClose}
        >Cancel</Button
      >
      <Button type="submit" size="small" disabled={!name.trim() || saving}>Save</Button>
    </div>
  </form>
</Dialog>
