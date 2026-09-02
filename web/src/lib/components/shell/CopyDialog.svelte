<script lang="ts">
  import CheckboxRow from '$lib/components/CheckboxRow.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import { copyDialog } from '$lib/stores/copyDialog.svelte';
  import { Button } from '@immich/ui';
  import {
    DEVELOP_KEYS,
    SECTION_LABELS,
    allSections,
    allSelected,
    hasSelectedSections,
    type SectionKey
  } from '$lib/copyPaste';

  function set(key: SectionKey, value: boolean): void {
    copyDialog.sections = { ...copyDialog.sections, [key]: value };
  }

  function setDevelop(value: boolean): void {
    const next = { ...copyDialog.sections };
    for (const k of DEVELOP_KEYS) next[k] = value;
    copyDialog.sections = next;
  }

  const developAll = $derived(allSelected(copyDialog.sections, DEVELOP_KEYS));
  const canCopy = $derived(hasSelectedSections(copyDialog.sections));
</script>

{#if copyDialog.open}
  <Dialog title="Copy settings" onClose={copyDialog.cancel}>
    {#snippet actions()}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        onclick={() => {
          copyDialog.sections = allSections(true);
        }}
      >
        Check all
      </Button>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        onclick={() => {
          copyDialog.sections = allSections(false);
        }}
      >
        Check none
      </Button>
    {/snippet}

    {#snippet footer()}
      <div class="flex w-full justify-end gap-2">
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="bg-light-100 hover:bg-light-200"
          onclick={copyDialog.cancel}
        >
          Cancel
        </Button>
        <Button size="tiny" color="primary" disabled={!canCopy} onclick={copyDialog.confirm}>
          Copy
        </Button>
      </div>
    {/snippet}

    <div class="flex flex-col gap-1 text-xs text-dark/80">
      <CheckboxRow
        label="Develop"
        class="py-0.5 text-xs font-medium"
        checked={developAll}
        onChange={setDevelop}
      />
      <div class="flex flex-col gap-0.5 pl-6">
        {#each DEVELOP_KEYS as key (key)}
          <CheckboxRow
            label={SECTION_LABELS[key]}
            class="py-0.5 text-xs"
            checked={copyDialog.sections[key]}
            onChange={(v) => set(key, v)}
          />
        {/each}
      </div>

      <div class="border-t border-dark/10 mt-2 pt-2 flex flex-col gap-0.5">
        <CheckboxRow
          label={SECTION_LABELS.geometry}
          class="py-0.5 text-xs"
          checked={copyDialog.sections.geometry}
          onChange={(v) => set('geometry', v)}
        />
        <CheckboxRow
          label={SECTION_LABELS.masks}
          class="py-0.5 text-xs"
          checked={copyDialog.sections.masks}
          onChange={(v) => set('masks', v)}
        />
        <CheckboxRow
          label={SECTION_LABELS.retouch}
          class="py-0.5 text-xs"
          checked={copyDialog.sections.retouch}
          onChange={(v) => set('retouch', v)}
        />
      </div>
    </div>
  </Dialog>
{/if}
