<script lang="ts">
  import { versionSummary } from '#lib/editor/versions.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { untrack } from 'svelte';
  import NewVersionRow from './NewVersionRow.svelte';
  import VersionRow from './VersionRow.svelte';

  let { onnavigate = () => {} }: { onnavigate?: () => void } = $props();

  const store = editor.versions;

  let editingId = $state<string | null>(null);

  $effect(() => {
    if (editor.assetId) untrack(() => void store.load());
  });
</script>

<nav aria-label="Versions" class="flex flex-col">
  <span class="section-label px-3 pt-1.5 pb-1 text-dark/65 touch:hidden">Versions</span>
  {#each store.items as version (version.id)}
    <VersionRow
      {version}
      current={store.current?.id === version.id}
      editing={editingId === version.id}
      summary={versionSummary(version, store.summaries.get(version.id), editor.isRaw)}
      onrename={() => (editingId = version.id)}
      onendrename={() => (editingId = null)}
      {onnavigate}
    />
  {/each}
  <NewVersionRow onrun={onnavigate} />
</nav>
