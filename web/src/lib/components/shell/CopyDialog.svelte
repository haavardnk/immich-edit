<script lang="ts">
  import { copyDialog } from '$lib/stores/copyDialog.svelte';
  import {
    DEVELOP_KEYS,
    SECTION_LABELS,
    allSections,
    allSelected,
    hasSelectedSections,
    type SectionKey
  } from '$lib/copyPaste';

  function onBackdropClick(e: MouseEvent): void {
    if (e.currentTarget === e.target) copyDialog.cancel();
  }

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
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={onBackdropClick}
  >
    <div
      class="bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl p-5 min-w-[320px] max-w-105"
      role="dialog"
      aria-modal="true"
      aria-label="Copy settings"
    >
      <div class="flex items-center justify-between mb-3">
        <h2 class="text-sm font-medium text-immich-dark-fg">Copy settings</h2>
        <div class="flex gap-1.5">
          <button
            class="text-[11px] px-2 py-0.5 rounded text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
            onclick={() => {
              copyDialog.sections = allSections(true);
            }}
          >
            Check all
          </button>
          <button
            class="text-[11px] px-2 py-0.5 rounded text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
            onclick={() => {
              copyDialog.sections = allSections(false);
            }}
          >
            Check none
          </button>
        </div>
      </div>

      <div class="flex flex-col gap-1 text-xs text-immich-dark-fg/80">
        <label class="flex items-center gap-2 py-0.5 font-medium">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            checked={developAll}
            onchange={(e) => setDevelop(e.currentTarget.checked)}
          />
          Develop
        </label>
        <div class="flex flex-col gap-0.5 pl-6">
          {#each DEVELOP_KEYS as key (key)}
            <label class="flex items-center gap-2 py-0.5">
              <input
                type="checkbox"
                class="checkbox checkbox-xs"
                checked={copyDialog.sections[key]}
                onchange={(e) => set(key, e.currentTarget.checked)}
              />
              {SECTION_LABELS[key]}
            </label>
          {/each}
        </div>

        <div class="border-t border-white/10 mt-2 pt-2 flex flex-col gap-0.5">
          <label class="flex items-center gap-2 py-0.5">
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              checked={copyDialog.sections.geometry}
              onchange={(e) => set('geometry', e.currentTarget.checked)}
            />
            {SECTION_LABELS.geometry}
          </label>
          <label class="flex items-center gap-2 py-0.5">
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              checked={copyDialog.sections.masks}
              onchange={(e) => set('masks', e.currentTarget.checked)}
            />
            {SECTION_LABELS.masks}
          </label>
          <label class="flex items-center gap-2 py-0.5">
            <input
              type="checkbox"
              class="checkbox checkbox-xs"
              checked={copyDialog.sections.retouch}
              onchange={(e) => set('retouch', e.currentTarget.checked)}
            />
            {SECTION_LABELS.retouch}
          </label>
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-4">
        <button
          class="text-xs px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 transition-colors"
          onclick={copyDialog.cancel}
        >
          Cancel
        </button>
        <button
          class="text-xs px-3 py-1.5 rounded-lg bg-immich-dark-primary/20 text-immich-dark-primary hover:bg-immich-dark-primary/30 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!canCopy}
          onclick={copyDialog.confirm}
        >
          Copy
        </button>
      </div>
    </div>
  </div>
{/if}
