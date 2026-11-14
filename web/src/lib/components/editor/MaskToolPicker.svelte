<script lang="ts">
  import { Button, Icon } from '@immich/ui';
  import { mdiChevronDown, mdiChevronUp, mdiDownloadOutline } from '@mdi/js';
  import type { MaskKind, SemanticClass } from '#lib/api/masks.ts';
  import { aiTools, MANUAL_TOOLS, type ToolPick } from '#lib/panels/maskToolCatalog.ts';
  import { visibleSceneClasses } from '#lib/types/masks.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';

  let {
    aiKinds,
    semanticClasses = [],
    aiUnavailable = null,
    busy = false,
    onpick
  }: {
    aiKinds: { kind: MaskKind; installed: boolean }[];
    semanticClasses?: SemanticClass[];
    aiUnavailable?: string | null;
    busy?: boolean;
    onpick: (pick: ToolPick) => void;
  } = $props();

  let semanticOpen = $state(false);

  const sceneClasses = $derived(visibleSceneClasses(semanticClasses, aiKinds));
  const tools = $derived(aiTools(aiKinds));
</script>

<div class="w-56 py-1">
  {#if tools.length > 0}
    <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">AI</div>
    {#each tools as entry (entry.id)}
      <Button
        type="button"
        size="tiny"
        variant="ghost"
        color="secondary"
        class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left {entry.installed
          ? ''
          : 'text-dark/65'}"
        disabled={busy && entry.installed && entry.generates}
        aria-label={entry.label}
        onclick={() => {
          if (entry.scene) semanticOpen = !semanticOpen;
          else onpick(entry.pick);
        }}
      >
        <Icon icon={entry.icon} size="14px" class="mt-0.5 shrink-0 opacity-70" aria-hidden="true" />
        <span class="flex-1 min-w-0">
          <span class="block text-xs truncate">{entry.label}</span>
          <span class="block text-[10px] text-dark/65 truncate">{entry.hint}</span>
        </span>
        {#if !entry.installed}
          <Icon icon={mdiDownloadOutline} size="13px" class="mt-0.5 shrink-0" aria-hidden="true" />
        {:else if entry.scene}
          <Icon
            icon={semanticOpen ? mdiChevronUp : mdiChevronDown}
            size="13px"
            class="mt-0.5 shrink-0"
            aria-hidden="true"
          />
        {/if}
      </Button>
      {#if entry.scene && semanticOpen}
        {#each sceneClasses as cls (cls.id)}
          <Button
            type="button"
            size="tiny"
            variant="ghost"
            color="secondary"
            class="w-full justify-start gap-2.5 rounded-none py-1 pl-10 pr-3 text-left text-xs"
            disabled={busy}
            onclick={() =>
              onpick({ kind: 'ai', mask: 'semantic', installed: true, maskClass: cls.id })}
          >
            {cls.name}
          </Button>
        {/each}
      {/if}
    {/each}
    <div class="my-1 border-t border-dark/10"></div>
  {:else if aiUnavailable}
    <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">AI</div>
    <div class="px-3 pb-2 text-[10px] text-dark/65 leading-snug">
      {aiUnavailable}
    </div>
    <div class="my-1 border-t border-dark/10"></div>
  {/if}
  <div class="px-3 pt-1 pb-1 text-[10px] uppercase tracking-wider text-dark/65">Manual</div>
  {#each MANUAL_TOOLS as item (item.tool)}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="w-full items-start justify-start gap-2.5 rounded-none px-3 py-1.5 text-left"
      aria-label={item.label}
      onclick={() => onpick({ kind: 'manual', tool: item.tool })}
    >
      <Icon icon={item.icon} size="14px" class="mt-0.5 shrink-0 opacity-70" aria-hidden="true" />
      <span class="flex-1 min-w-0">
        <span class="block text-xs truncate">{item.label}</span>
        <span class="block text-[10px] text-dark/65 truncate">{item.hint}</span>
      </span>
      {#if item.bind}
        <span class="mt-0.5 shrink-0 font-mono text-[10px] text-dark/65">{keysFor(item.bind)}</span>
      {/if}
    </Button>
  {/each}
</div>
