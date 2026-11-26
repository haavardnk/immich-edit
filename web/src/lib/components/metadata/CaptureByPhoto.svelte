<script lang="ts">
  import { thumbUrl } from '#lib/api/assets.ts';
  import WallTimeField from '#lib/components/metadata/WallTimeField.svelte';
  import { shownText } from '#lib/metadata/captureLabels.ts';
  import { correctionText } from '#lib/metadata/clockText.ts';
  import { ALL_CAMERAS, captureEditor } from '#lib/stores/captureEditor.svelte.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiChevronLeft, mdiChevronRight } from '@mdi/js';

  const editor = captureEditor;
  const picked = $derived(editor.picked);
  const match = $derived(editor.camera !== ALL_CAMERAS && editor.references.length > 0);
  const matched = $derived(editor.photos.find((photo) => photo.id === editor.reference) ?? null);
</script>

{#if picked}
  <div
    class="flex flex-col overflow-hidden rounded-xl bg-light-100 ring-1 ring-hairline @3xl:min-h-28"
  >
    <div class="relative aspect-[3/2] min-h-0 bg-image-canvas">
      <img
        src={thumbUrl(picked.id, 'preview')}
        alt={picked.name}
        class="absolute inset-0 size-full object-contain"
      />
      <IconButton
        size="small"
        shape="round"
        variant="filled"
        color="secondary"
        class="absolute start-2 top-1/2 -translate-y-1/2 touch:size-11"
        icon={mdiChevronLeft}
        title="Previous photo"
        aria-label="Previous photo"
        disabled={editor.pickable.length < 2}
        onclick={() => editor.step(-1)}
      />
      <IconButton
        size="small"
        shape="round"
        variant="filled"
        color="secondary"
        class="absolute end-2 top-1/2 -translate-y-1/2 touch:size-11"
        icon={mdiChevronRight}
        title="Next photo"
        aria-label="Next photo"
        disabled={editor.pickable.length < 2}
        onclick={() => editor.step(1)}
      />
    </div>
    <div class="flex items-center gap-2 px-3 py-2 text-xs text-dark/65 touch:text-sm">
      <span class="min-w-0 flex-1 truncate">{picked.name}</span>
      <span class="shrink-0 tabular-nums">Shows {shownText(picked.wall, true)}</span>
    </div>
  </div>
  <div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
    <span class="min-w-0 flex-1 text-sm text-light-950">Actual time</span>
    <WallTimeField
      label="Actual time"
      value={editor.shownOf(picked)}
      onChange={(wall) => editor.setPickedTime(wall)}
    />
  </div>
  {#if match}
    <div class="flex items-center gap-2 text-caption text-dark/65" aria-hidden="true">
      <span class="h-px flex-1 bg-hairline"></span>
      or match a photo from another camera
      <span class="h-px flex-1 bg-hairline"></span>
    </div>
    <div
      class="flex shrink-0 gap-1.5 overflow-x-auto pb-1"
      role="group"
      aria-label="Photos from other cameras"
    >
      {#each editor.references as reference (reference.id)}
        <Button
          size="tiny"
          variant="ghost"
          color={editor.reference === reference.id ? 'primary' : 'secondary'}
          class="h-auto shrink-0 flex-col gap-1 p-1 -outline-offset-2 {editor.reference ===
          reference.id
            ? 'bg-primary/10 ring-2 ring-primary ring-inset'
            : ''}"
          title={reference.name}
          aria-label="Match to {reference.name} from {editor.labelOf(reference)}"
          aria-pressed={editor.reference === reference.id}
          onclick={() => editor.matchTo(reference.id)}
        >
          <img
            src={thumbUrl(reference.id, 'thumbnail')}
            alt=""
            loading="lazy"
            class="h-12 w-16 rounded object-cover touch:h-14 touch:w-20"
          />
          <span class="w-16 truncate text-caption text-dark/75 touch:w-20">
            {editor.labelOf(reference)}
          </span>
          <span class="text-caption tabular-nums">
            {shownText(editor.shownAsPicked(reference), false)}
          </span>
        </Button>
      {/each}
    </div>
  {/if}
  {#if editor.camera !== ALL_CAMERAS && (editor.clocks.of(picked.camera) !== 0 || matched)}
    <p class="rounded-lg bg-primary/10 px-3 py-2 text-xs text-light-950 touch:text-sm">
      {editor.labelOf(picked)}
      {correctionText(editor.clocks.of(picked.camera))}
      {#if matched}
        <span class="block">Matched to {matched.name} from {editor.labelOf(matched)}</span>
      {/if}
    </p>
  {/if}
{/if}
