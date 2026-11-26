<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { onNavigate } from '$app/navigation';
  import type { Snippet } from 'svelte';
  import DesktopBrowseShell from '#lib/components/shell/desktop/DesktopBrowseShell.svelte';
  import TouchBrowseShell from '#lib/components/shell/touch/TouchBrowseShell.svelte';
  import WideBrowseShell from '#lib/components/shell/wide/WideBrowseShell.svelte';
  import KeybindsHelp from './shortcuts/KeybindsHelp.svelte';
  import CopyDialog from './CopyDialog.svelte';
  import MetadataConsentDialog from './MetadataConsentDialog.svelte';
  import TrashDialog from './TrashDialog.svelte';
  import SignOutConfirm from './SignOutConfirm.svelte';
  import JobsOverlay from '#lib/components/jobs/JobsOverlay.svelte';
  import JobsTabStatus from '#lib/components/jobs/JobsTabStatus.svelte';
  import AlbumConfirmDialog from '#lib/components/library/AlbumConfirmDialog.svelte';
  import AlbumDetailsDialog from '#lib/components/library/AlbumDetailsDialog.svelte';
  import TagConfirmDialog from '#lib/components/library/TagConfirmDialog.svelte';
  import TagDetailsDialog from '#lib/components/library/TagDetailsDialog.svelte';
  import LocationPickerDialog from '#lib/components/map/LocationPickerDialog.svelte';
  import CaptureTimeDialog from '#lib/components/metadata/CaptureTimeDialog.svelte';
  import DescriptionDialog from '#lib/components/metadata/DescriptionDialog.svelte';
  import Loupe from '#lib/components/browse/Loupe.svelte';
  import PeekView from '#lib/components/browse/PeekView.svelte';
  import { peek } from '#lib/browse/peek.svelte.ts';
  import { browsing } from '#lib/stores/browsing.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { leaveLoupeOnNavigate } from '#lib/browse/loupeActions.ts';
  import { zoomOut } from '#lib/browse/loupeZoom.ts';
  import { enteringEditor, handoffId, holdPhoto } from '#lib/browse/photoHandoff.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { onEditorRoute } from '#lib/utils/route.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { animatePageChange } from '#lib/navigation/pageTransition.ts';

  let { children }: { children: Snippet } = $props();

  const onEditor = $derived(onEditorRoute());
  const peeked = $derived(peek.id ? browsing.find(peek.id) : null);

  let loupeShown: string | null = null;

  onMount(() => {
    void editedThumbs.loadOnce();
    void jobs.load();
  });

  onNavigate((navigation) => {
    peek.hide();
    holdPhoto(handoffId(navigation.from?.url, navigation.to?.url));
    if (navigation.to) {
      leaveLoupeOnNavigate(navigation.from?.url, navigation.to.url);
    }
    if (enteringEditor(navigation.from?.url, navigation.to?.url)) {
      ui.startEditing(layout.mode === 'touch');
      touchUi.startDevelop();
    }
    return animatePageChange(navigation, layout.compact);
  });

  $effect.pre(() => {
    browseView.setCompact(layout.compact);
  });

  $effect.pre(() => {
    const id = browseView.loupeId;
    const closed = loupeShown;
    loupeShown = id;
    if (closed && !id && untrack(() => browseView.editorReturnLoupeId) === null) zoomOut(closed);
  });

  $effect(() => {
    document.documentElement.classList.toggle('grey-canvas', ui.greyCanvas);
  });
</script>

<div class="contents" inert={browseView.loupeId !== null && !browseView.loupeBehindEditor}>
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
<AlbumDetailsDialog />
<AlbumConfirmDialog />
<TagDetailsDialog />
<TagConfirmDialog />
<LocationPickerDialog />
<CaptureTimeDialog />
<DescriptionDialog />
<JobsOverlay />
<JobsTabStatus />
<SignOutConfirm />
{#if browseView.loupeId}
  <Loupe />
{/if}
{#if peeked}
  <PeekView asset={peeked} />
{/if}
