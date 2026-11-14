<script lang="ts">
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import type { Version } from '#lib/editor/versions.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { longPressRecognizer } from '#lib/gestures/longPress.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Icon } from '@immich/ui';
  import { mdiContentDuplicate } from '@mdi/js';

  const HOLD_MS = 250;

  let {
    version,
    current,
    onhold = () => {}
  }: { version: Version; current: boolean; onhold?: () => void } = $props();

  const hold = editor.versions.hold;
  const src = $derived(
    assetThumbUrl({
      id: version.id,
      thumbhash: editor.asset?.thumbhash,
      checksum: editor.asset?.checksum
    })
  );

  const press = gestures({
    recognizers: [
      longPressRecognizer({
        thresholds: { longPressMs: HOLD_MS },
        enabled: () => hold.canHold(version.id),
        start: () => {
          onhold();
          hold.start(version);
        },
        end: () => hold.end(version.id),
        cancel: () => hold.end(version.id)
      })
    ],
    touchAction: null
  });

  $effect(() => () => hold.end(version.id));
</script>

<span
  class="relative h-6 w-8 shrink-0 overflow-hidden rounded bg-light-200 touch:size-14 touch:rounded-lg {current
    ? 'touch:ring-2 touch:ring-primary touch:ring-offset-2 touch:ring-offset-light-50'
    : ''}"
  data-version-thumb
  {@attach press}
>
  <img {src} alt="" draggable="false" class="no-callout size-full object-cover select-none" />
  {#if version.copyIndex !== null}
    <span
      class="absolute right-1 bottom-1 hidden size-5 items-center justify-center rounded bg-black/60 text-white touch:flex"
      aria-hidden="true"
    >
      <Icon icon={mdiContentDuplicate} size="13px" />
    </span>
  {/if}
</span>
