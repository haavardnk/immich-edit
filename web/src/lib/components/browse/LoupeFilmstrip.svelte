<script lang="ts">
  import ContextMenuItem from '#lib/components/ContextMenuItem.svelte';
  import Filmstrip from '#lib/components/shell/Filmstrip.svelte';
  import { openInEditor } from '#lib/browse/loupeActions.ts';
  import { pickFromStrip } from '#lib/browse/loupeNav.ts';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { compare } from '#lib/stores/compare.svelte.ts';

  let {
    focusedId,
    currentId,
    multi
  }: { focusedId: string | null; currentId: string | null; multi: boolean } = $props();
</script>

{#snippet stripMenu(id: string)}
  {@const paneAction = multi && compare.members.includes(id) ? 'Remove from' : 'Add to'}
  {@const paneLabel = `${paneAction} ${compare.mode === 'survey' ? 'survey' : 'compare'}`}
  <ContextMenuItem title={hint('Open in editor', 'openEditor')} onSelect={() => openInEditor(id)}>
    Open in editor
  </ContextMenuItem>
  <ContextMenuItem disabled={!multi && id === currentId} onSelect={() => pickFromStrip(id, true)}>
    {paneLabel}
  </ContextMenuItem>
{/snippet}

<Filmstrip
  currentId={focusedId}
  highlightIds={compare.members}
  onSelect={pickFromStrip}
  resizable
  size={72}
  showBadges
  collapsed={!browseView.loupeFilmstripShown}
  menu={stripMenu}
/>
