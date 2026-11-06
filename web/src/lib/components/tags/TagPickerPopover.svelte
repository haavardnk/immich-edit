<script lang="ts">
  import { isManagedTag, userTags } from '#lib/browse/managedTags.ts';
  import Popover from '#lib/components/Popover.svelte';
  import TagPickerBody from '#lib/components/tags/TagPickerBody.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { toasts } from '#lib/stores/toasts.svelte.ts';
  import type { TagRef } from '#lib/types/asset.ts';
  import { onMount, type Snippet } from 'svelte';

  type Anchor = 'top' | 'bottom';
  type Align = 'start' | 'end' | 'center';

  let {
    tags,
    open,
    onToggle,
    onClose,
    onAdd,
    onRemove,
    onCreate,
    trigger,
    disabled = false,
    anchor = 'top',
    align = 'start',
    subtitle,
    tooltip,
    showApplied = false
  }: {
    tags: TagRef[];
    open: boolean;
    onToggle: () => void;
    onClose: () => void;
    onAdd: (tag: TagRef) => void | Promise<void>;
    onRemove: (tagId: string) => void | Promise<void>;
    onCreate: (value: string) => TagRef | null | Promise<TagRef | null>;
    trigger: Snippet<[Record<string, unknown>]>;
    disabled?: boolean;
    anchor?: Anchor;
    align?: Align;
    subtitle?: string;
    tooltip?: string;
    showApplied?: boolean;
  } = $props();

  onMount(() => {
    library.ensureTags().catch((e: unknown) => toasts.fail('tags', e));
  });

  const sheet = $derived(layout.mode === 'touch');
  const allTags = $derived(userTags(library.tags ?? []));
  const visibleTags = $derived(tags.filter((t) => !isManagedTag(t)));
</script>

<Popover
  open={open && !disabled}
  {anchor}
  {align}
  onOpenChange={(v) => (v ? onToggle() : onClose())}
  appearance="panel"
  contentClass="flex w-72 flex-col max-h-[min(28rem,var(--bits-popover-content-available-height))]!"
  sheetClass="flex flex-col"
  sheetPicker
  title="Tags"
  {subtitle}
  {tooltip}
  {trigger}
>
  <TagPickerBody
    all={allTags}
    applied={visibleTags}
    showApplied={sheet || showApplied}
    numbered={!sheet || layout.hasKeyboard}
    {disabled}
    autofocus={!sheet}
    {onAdd}
    {onRemove}
    {onCreate}
  />
</Popover>
