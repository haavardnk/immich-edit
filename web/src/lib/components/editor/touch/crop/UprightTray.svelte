<script lang="ts">
  import GuideCounts from '#lib/components/editor/crop/GuideCounts.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { GeometrySession } from '#lib/stores/editor/geometry.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { guideStatus } from '#lib/utils/uprightFeedback.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiDeleteOutline, mdiRulerSquare } from '@mdi/js';

  let { sess }: { sess: GeometrySession } = $props();

  const status = $derived(guideStatus(sess.guides, sess.guideNote));
</script>

<section aria-label="Guided upright" class="flex shrink-0 flex-col gap-2 px-4 pt-2 pb-1">
  <div class="flex items-center gap-3">
    <Icon icon={mdiRulerSquare} size="24px" class="shrink-0 text-primary" aria-hidden="true" />
    <div class="min-w-0">
      <p class="text-[15px] font-semibold text-light-950">Guided upright</p>
      <p class="text-[12.5px] text-dark/70"><GuideCounts guides={sess.guides} /></p>
    </div>
  </div>
  {#each status as line (line.text)}
    <p
      class="text-[13px] leading-snug {line.tone === 'warn' ? 'text-warning-800' : 'text-dark/80'}"
      role="status"
    >
      {line.text}
    </p>
  {/each}
  <div class="flex items-center justify-between gap-2 pt-1">
    <Button
      size="small"
      variant="ghost"
      color="secondary"
      shape="round"
      class="bg-ghost"
      leadingIcon={mdiDeleteOutline}
      disabled={sess.guides.length === 0}
      onclick={editor.clearUprightGuides}>Clear guides</Button
    >
    <Button size="small" color="primary" shape="round" onclick={() => (ui.geometryTool = null)}
      >Done drawing</Button
    >
  </div>
</section>
