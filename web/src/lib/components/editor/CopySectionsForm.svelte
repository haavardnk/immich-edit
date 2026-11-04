<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { DEVELOP_KEYS, SECTION_LABELS, allSelected } from '#lib/editor/copyPaste.ts';

  const developAll = $derived(allSelected(copyDialog.sections, DEVELOP_KEYS));
</script>

<div class="flex flex-col gap-1 text-xs text-dark/80">
  <CheckboxRow
    label="Develop"
    class="py-0.5 text-xs font-medium"
    checked={developAll}
    onChange={copyDialog.setDevelop}
  />
  <div class="flex flex-col gap-0.5 pl-6">
    {#each DEVELOP_KEYS as key (key)}
      <CheckboxRow
        label={SECTION_LABELS[key]}
        class="py-0.5 text-xs"
        checked={copyDialog.sections[key]}
        onChange={(v) => copyDialog.setSection(key, v)}
      />
    {/each}
  </div>

  <div class="border-t border-dark/10 mt-2 pt-2 flex flex-col gap-0.5">
    <CheckboxRow
      label={SECTION_LABELS.geometry}
      class="py-0.5 text-xs"
      checked={copyDialog.sections.geometry}
      onChange={(v) => copyDialog.setSection('geometry', v)}
    />
    <CheckboxRow
      label={SECTION_LABELS.masks}
      class="py-0.5 text-xs"
      checked={copyDialog.sections.masks}
      onChange={(v) => copyDialog.setSection('masks', v)}
    />
    <CheckboxRow
      label={SECTION_LABELS.retouch}
      class="py-0.5 text-xs"
      checked={copyDialog.sections.retouch}
      onChange={(v) => copyDialog.setSection('retouch', v)}
    />
  </div>
</div>
