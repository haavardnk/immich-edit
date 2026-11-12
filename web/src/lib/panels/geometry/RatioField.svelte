<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiBookmarkOutline,
    mdiBookmarkRemoveOutline,
    mdiCropLandscape,
    mdiCropPortrait
  } from '@mdi/js';
  import { aspectText, parseRatio, ratioReadout } from '#lib/panels/geometry/aspects.ts';
  import {
    applyTyped,
    canFlipOrientation,
    draftAspect,
    draftPortrait,
    isSavedRatio,
    savableRatio,
    setOrientation,
    toggleSavedRatio
  } from '#lib/panels/geometry/ratioActions.ts';

  let typed = $state<string | null>(null);

  const committed = $derived(aspectText(draftAspect()));
  const text = $derived(typed ?? committed);
  const parsed = $derived(parseRatio(text));
  const invalid = $derived(typed !== null && typed.trim() !== '' && parsed === null);
  const portrait = $derived(draftPortrait());
  const savable = $derived(savableRatio());
  const saved = $derived(savable !== null && isSavedRatio(savable));

  function commit(): void {
    if (typed === null) return;
    if (typed.trim() === '' || applyTyped(typed)) typed = null;
  }

  function onKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      commit();
    } else if (e.key === 'Escape' && typed !== null) {
      e.preventDefault();
      e.stopPropagation();
      typed = null;
    }
  }
</script>

<div class="flex items-center gap-1.5">
  <TextInput
    compact
    color="neutral"
    class="min-w-0 flex-1 [&_input]:h-7 [&_input]:px-2"
    aria-label="Ratio"
    aria-invalid={invalid}
    placeholder="21:9 or 2.39"
    inputmode="decimal"
    trailingText={parsed ? `= ${ratioReadout(parsed)}` : undefined}
    value={text}
    oninput={(e) => (typed = e.currentTarget.value)}
    onkeydown={onKeydown}
    onblur={commit}
  />
  <IconButton
    size="tiny"
    variant="ghost"
    color="secondary"
    class="size-7 shrink-0 bg-light-200 not-disabled:hover:bg-light-300"
    icon={portrait ? mdiCropPortrait : mdiCropLandscape}
    title={portrait ? 'Switch to landscape' : 'Switch to portrait'}
    aria-label={portrait ? 'Switch to landscape' : 'Switch to portrait'}
    disabled={!canFlipOrientation()}
    onclick={() => setOrientation(!portrait)}
  />
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-7 shrink-0 bg-light-200 px-2 not-disabled:hover:bg-light-300"
    leadingIcon={saved ? mdiBookmarkRemoveOutline : mdiBookmarkOutline}
    disabled={savable === null}
    onclick={toggleSavedRatio}
  >
    {saved ? 'Remove ratio' : 'Save ratio'}
  </Button>
</div>
