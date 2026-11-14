<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import GuideCounts from '#lib/components/editor/crop/GuideCounts.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { GeometrySession } from '#lib/stores/editor/geometry.svelte.ts';
  import { guideStatus } from '#lib/utils/uprightFeedback.ts';
  import { Button } from '@immich/ui';
  import { mdiClose } from '@mdi/js';

  let { sess }: { sess: GeometrySession } = $props();

  const status = $derived(guideStatus(sess.guides, sess.guideNote));
</script>

<div class="flex flex-col gap-1.5 rounded-lg bg-light-100 p-2" data-testid="upright-card">
  <div class="flex items-center justify-between gap-2">
    <span class="text-xs font-semibold">Guided upright</span>
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      leadingIcon={mdiClose}
      disabled={sess.guides.length === 0}
      onclick={editor.clearUprightGuides}>Clear guides</Button
    >
  </div>
  <div class="text-[11px] text-dark/80"><GuideCounts guides={sess.guides} /></div>
  {#each status as line (line.text)}
    <Notice color={line.tone === 'warn' ? 'warning' : 'info'} message={line.text} />
  {/each}
</div>
