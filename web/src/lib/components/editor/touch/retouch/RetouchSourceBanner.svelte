<script lang="ts">
  import { SOURCE_MISSING_TEXT } from '#lib/editor/retouch/text.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiClose, mdiCrosshairsGps } from '@mdi/js';

  const sampling = $derived(editor.retouchSampling);
  const missed = $derived(editor.retouchSourceMissed && !editor.retouchAnchor);
</script>

{#if sampling || missed}
  <div
    role="status"
    data-testid="retouch-source-banner"
    class="absolute inset-x-3 top-3 z-20 flex items-center gap-3 rounded-xl bg-black/70 py-2 ps-3 pe-1.5 text-white shadow-lg backdrop-blur-sm [[data-arrangement=side]_&]:top-15"
  >
    <Icon
      icon={mdiCrosshairsGps}
      size="20px"
      class="shrink-0 text-retouch-source"
      aria-hidden="true"
    />
    <div class="flex min-w-0 flex-1 flex-col">
      <span class="truncate text-[14px] font-medium">
        {sampling ? 'Tap the photo to set the source' : SOURCE_MISSING_TEXT}
      </span>
      <span class="truncate text-[12px] text-white/70">
        {sampling ? 'Pick clean texture, then paint over the spot' : 'Tap Set source first'}
      </span>
    </div>
    {#if sampling}
      <IconButton
        size="medium"
        variant="ghost"
        color="secondary"
        class="size-10 shrink-0 text-white"
        icon={mdiClose}
        aria-label="Stop picking the source"
        onclick={editor.toggleRetouchSampling}
      />
    {:else}
      <Button
        type="button"
        size="small"
        shape="round"
        color="primary"
        class="shrink-0 px-4"
        onclick={editor.toggleRetouchSampling}
      >
        Set source
      </Button>
    {/if}
  </div>
{/if}
