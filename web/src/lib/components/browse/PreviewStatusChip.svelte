<script lang="ts">
  import { previewStatusLabel } from '#lib/browse/previewSource.ts';
  import { Icon } from '@immich/ui';
  import { mdiLoading } from '@mdi/js';

  const SHOW_AFTER_MS = 400;

  let { pending, edited, episode }: { pending: boolean; edited: boolean; episode: string } =
    $props();

  let late = $state(false);

  $effect(() => {
    if (!pending) return;
    void episode;
    const timer = setTimeout(() => (late = true), SHOW_AFTER_MS);
    return () => {
      clearTimeout(timer);
      late = false;
    };
  });
</script>

<div class="pointer-events-none absolute bottom-3 left-3 z-10" role="status" aria-live="polite">
  {#if pending && late}
    <span
      data-testid="preview-status"
      class="flex items-center gap-1.5 rounded-full bg-black/70 px-2.5 py-1 text-[11px] text-white/85 shadow-lg backdrop-blur-sm"
    >
      <Icon icon={mdiLoading} size="12px" class="animate-spin" aria-hidden="true" />
      {previewStatusLabel(edited)}
    </span>
  {/if}
</div>
