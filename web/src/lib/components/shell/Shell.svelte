<script lang="ts">
  import { onMount } from 'svelte';
  import { onNavigate } from '$app/navigation';
  import type { Snippet } from 'svelte';
  import DesktopBrowseShell from '#lib/components/shell/desktop/DesktopBrowseShell.svelte';
  import TouchBrowseShell from '#lib/components/shell/touch/TouchBrowseShell.svelte';
  import WideBrowseShell from '#lib/components/shell/wide/WideBrowseShell.svelte';
  import KeybindsHelp from './KeybindsHelp.svelte';
  import CopyDialog from './CopyDialog.svelte';
  import MetadataConsentDialog from './MetadataConsentDialog.svelte';
  import TrashDialog from './TrashDialog.svelte';
  import SignOutConfirm from './SignOutConfirm.svelte';
  import JobsDrawer from '#lib/components/jobs/JobsDrawer.svelte';
  import Loupe from '#lib/components/browse/Loupe.svelte';
  import PeekView from '#lib/components/browse/PeekView.svelte';
  import { peek } from '#lib/browse/peek.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { leaveLoupeOnEditorPath } from '#lib/browse/loupeActions.ts';
  import { onEditorRoute } from '#lib/utils/route.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';

  let { children }: { children: Snippet } = $props();

  const onEditor = $derived(onEditorRoute());
  const peeked = $derived(peek.id ? browsing.find(peek.id) : null);

  onMount(() => {
    void editedThumbs.loadOnce();
    void jobs.load();
  });

  onNavigate(({ to }) => {
    peek.hide();
    if (to) leaveLoupeOnEditorPath(to.url.pathname);
  });

  $effect.pre(() => {
    browseView.setCompact(layout.compact);
  });

  $effect(() => {
    document.documentElement.classList.toggle('grey-canvas', ui.greyCanvas);
  });
</script>

<div class="contents" inert={browseView.loupeId !== null}>
  {#if onEditor}
    {@render children()}
  {:else if layout.compact}
    <TouchBrowseShell>
      {@render children()}
    </TouchBrowseShell>
  {:else if layout.wide}
    <WideBrowseShell>
      {@render children()}
    </WideBrowseShell>
  {:else}
    <DesktopBrowseShell>
      {@render children()}
    </DesktopBrowseShell>
  {/if}
</div>

<KeybindsHelp />
<CopyDialog />
<MetadataConsentDialog />
<TrashDialog />
<JobsDrawer />
<SignOutConfirm />
{#if browseView.loupeId}
  <Loupe />
{/if}
{#if peeked}
  <PeekView asset={peeked} />
{/if}
