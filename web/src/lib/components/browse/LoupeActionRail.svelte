<script lang="ts">
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import TagPicker from '#lib/components/TagPicker.svelte';
  import CullControls from '#lib/components/CullControls.svelte';
  import DisabledReason from '#lib/components/DisabledReason.svelte';
  import LoupePaneControls from '#lib/components/browse/LoupePaneControls.svelte';
  import LoupeViewTools from '#lib/components/browse/LoupeViewTools.svelte';
  import { NOT_OWNER } from '#lib/browse/ownership.ts';

  import type { CullActions, CullState } from '#lib/browse/cullState.ts';

  type Props = {
    cull: CullState;
    actions: CullActions;
    tags: TagRef[];
    fileName?: string;
    multi: boolean;
    zoom: number;
    fitZoom: number;
    fitMode: boolean;
    onAddTag: (tag: TagRef) => Promise<void>;
    onRemoveTag: (tagId: string) => Promise<void>;
    onCreateTag: (value: string) => Promise<TagRef | null>;
    onZoom: (value: number) => void;
    onFit: () => void;
  };

  let {
    cull,
    actions,
    tags,
    fileName,
    multi,
    zoom,
    fitZoom,
    fitMode,
    onAddTag,
    onRemoveTag,
    onCreateTag,
    onZoom,
    onFit
  }: Props = $props();
</script>

<nav
  aria-label="Photo actions"
  class="relative grid h-9 shrink-0 items-center border-t border-hairline bg-editor-chrome px-2 {multi
    ? 'grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]'
    : 'grid-cols-[minmax(0,1fr)_auto_auto]'}"
>
  <div class="min-w-0 overflow-hidden">
    <DisabledReason locked={!cull.owned} reason={NOT_OWNER} class="flex min-w-0 items-center gap-2">
      <CullControls {...cull} starSize={16} {actions} />
      <div class="flex min-w-0 flex-1">
        <TagPicker
          {tags}
          disabled={!cull.owned}
          open={browseView.loupeTagsOpen}
          onToggle={() => (browseView.loupeTagsOpen = !browseView.loupeTagsOpen)}
          onClose={() => (browseView.loupeTagsOpen = false)}
          onAdd={onAddTag}
          onRemove={onRemoveTag}
          onCreate={onCreateTag}
          anchor="bottom"
          subtitle={fileName}
        />
      </div>
    </DisabledReason>
  </div>
  <div class="flex items-center gap-1">
    {#if multi}
      <LoupePaneControls />
    {/if}
  </div>
  <div class="col-start-3 flex min-w-0 items-center justify-end gap-0.5">
    <LoupeViewTools {zoom} {fitZoom} {fitMode} {onZoom} {onFit} />
  </div>
</nav>
