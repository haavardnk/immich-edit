<script lang="ts">
  import { quickTools } from '#lib/panels/maskToolCatalog.ts';
  import { runPick } from '#lib/panels/maskTools.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskModels } from '#lib/stores/maskModels.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiViewGridOutline } from '@mdi/js';

  let { onall }: { onall?: () => void } = $props();

  const tools = $derived(quickTools(maskModels.kinds));
</script>

<section aria-label="New mask" class="flex flex-col gap-2 px-2 py-2 touch:gap-0 touch:p-0">
  <h3 class="hidden px-4 pt-3.5 pb-2 text-[15px] font-medium text-light-950 touch:block">
    New mask
  </h3>
  <div class="grid grid-cols-3 gap-1.5 touch:gap-2 touch:px-3">
    {#each tools as tool (tool.id)}
      <Button
        type="button"
        variant="ghost"
        color="secondary"
        class="h-14 flex-col gap-1 rounded-lg bg-ghost px-1 text-[11px] font-normal text-dark touch:h-19 touch:gap-1.5 touch:rounded-xl touch:text-[13px]"
        disabled={editor.maskGenerating}
        onclick={() => runPick({ layerId: null, mode: 'add' }, tool.pick)}
      >
        <Icon icon={tool.icon} size="20px" class="opacity-85" aria-hidden="true" />
        {tool.label}
      </Button>
    {/each}
  </div>
  {#if onall}
    <Button
      type="button"
      variant="outline"
      color="primary"
      class="mx-3 mt-2.5 mb-3 h-11 rounded-xl text-[14px] font-normal"
      leadingIcon={mdiViewGridOutline}
      onclick={onall}
    >
      All tools
    </Button>
  {/if}
</section>
