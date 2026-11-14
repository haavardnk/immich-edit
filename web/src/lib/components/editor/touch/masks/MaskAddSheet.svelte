<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { aiTools, MANUAL_TOOLS, type ToolPick } from '#lib/panels/maskToolCatalog.ts';
  import { runPick, type MaskTarget } from '#lib/panels/maskTools.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskModels } from '#lib/stores/maskModels.svelte.ts';
  import { visibleSceneClasses } from '#lib/types/masks.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronLeft, mdiChevronRight, mdiDownloadOutline } from '@mdi/js';

  let { target, onclose }: { target: MaskTarget | null; onclose: () => void } = $props();

  let scene = $state(false);
  let shown = $state<MaskTarget>({ layerId: null, mode: 'add' });

  $effect.pre(() => {
    if (target) shown = target;
  });

  const layerName = $derived(
    editor.edits.masks.find((layer) => layer.id === shown.layerId)?.name ?? ''
  );
  const title = $derived.by(() => {
    if (!shown.layerId) return 'New mask';
    if (shown.mode === 'subtract') return `Subtract from “${layerName}”`;
    if (shown.mode === 'intersect') return `Intersect with “${layerName}”`;
    return `Add to “${layerName}”`;
  });
  const subtitle = $derived.by(() => {
    if (!shown.layerId) return 'Pick a tool to start a mask.';
    if (shown.mode === 'subtract') return 'Pick a tool. Its area is cut out of the mask.';
    if (shown.mode === 'intersect') return 'Pick a tool. Only the overlap stays.';
    return 'Pick a tool. Its area joins the mask.';
  });
  const tools = $derived(aiTools(maskModels.kinds, true));
  const sceneClasses = $derived(visibleSceneClasses(maskModels.semanticClasses, maskModels.kinds));
  const heading = 'px-4 pt-3 pb-1.5 text-[11px] font-medium tracking-wider text-dark/65 uppercase';
  const cell =
    'h-auto w-full rounded-none border-hairline px-3.5 py-3 font-normal text-dark not-disabled:hover:bg-dark/5';

  function pick(choice: ToolPick): void {
    runPick(shown, choice);
  }
</script>

<ActionSheet
  open={target !== null}
  onOpenChange={(next) => {
    if (!next) onclose();
  }}
  onExited={() => (scene = false)}
  {title}
>
  <div
    class="flex flex-col items-center gap-0.5 border-b border-hairline px-4 pt-3.5 pb-3 text-center"
  >
    <span class="text-[15px] font-semibold" aria-hidden="true">{title}</span>
    <span class="text-[12.5px] text-dark/65">{subtitle}</span>
  </div>
  {#if scene}
    <MenuRow icon={mdiChevronLeft} label="Scene" data-menu-stay onclick={() => (scene = false)} />
    {#each sceneClasses as cls (cls.id)}
      <MenuRow
        label={cls.name}
        disabled={editor.maskGenerating}
        onclick={() => pick({ kind: 'ai', mask: 'semantic', installed: true, maskClass: cls.id })}
      />
    {/each}
  {:else}
    {#if tools.length > 0}
      <span class={heading}>AI</span>
      <div class="grid grid-cols-2 border-t border-hairline">
        {#each tools as entry, index (entry.id)}
          <Button
            type="button"
            variant="ghost"
            color="secondary"
            class="{cell} justify-start gap-3 border-b text-left {index % 2 === 0
              ? 'border-r'
              : ''}"
            aria-label={entry.label}
            disabled={editor.maskGenerating && entry.installed && entry.generates}
            data-menu-row
            data-menu-stay={entry.scene ? '' : undefined}
            onclick={() => {
              if (entry.scene) scene = true;
              else pick(entry.pick);
            }}
          >
            <Icon icon={entry.icon} size="20px" class="shrink-0 opacity-80" aria-hidden="true" />
            <span class="flex min-w-0 flex-1 flex-col">
              <span class="truncate text-[14px] {entry.installed ? '' : 'text-dark/65'}">
                {entry.label}
              </span>
              <span
                class="line-clamp-2 text-[11.5px] leading-tight {entry.installed
                  ? 'text-dark/65'
                  : 'text-warning'}"
              >
                {entry.hint}
              </span>
            </span>
            {#if !entry.installed}
              <Icon icon={mdiDownloadOutline} size="18px" class="shrink-0" aria-hidden="true" />
            {:else if entry.scene}
              <Icon
                icon={mdiChevronRight}
                size="18px"
                class="shrink-0 opacity-70"
                aria-hidden="true"
              />
            {/if}
          </Button>
        {/each}
      </div>
    {:else if maskModels.unavailable}
      <span class={heading}>AI</span>
      <p class="px-4 pb-3 text-[12.5px] leading-snug text-dark/65">{maskModels.unavailable}</p>
    {/if}
    <span class={heading}>Shapes</span>
    <div class="grid grid-cols-3 border-t border-hairline">
      {#each MANUAL_TOOLS as item, index (item.tool)}
        <Button
          type="button"
          variant="ghost"
          color="secondary"
          class="{cell} flex-col justify-center gap-1.5 text-center {index % 3 < 2
            ? 'border-r'
            : ''} {index < MANUAL_TOOLS.length - 3 ? 'border-b' : ''}"
          aria-label={item.label}
          data-menu-row
          onclick={() => pick({ kind: 'manual', tool: item.tool })}
        >
          <Icon icon={item.icon} size="20px" class="opacity-80" aria-hidden="true" />
          <span class="text-[12.5px] leading-tight">{item.label}</span>
        </Button>
      {/each}
    </div>
  {/if}
</ActionSheet>
