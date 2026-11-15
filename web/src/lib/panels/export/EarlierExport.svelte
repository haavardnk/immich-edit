<script lang="ts">
  import type { OnConflict } from '#lib/api/export.ts';
  import { segmented } from '#lib/components/segmented.ts';
  import { reveal } from '#lib/motion/reveal.ts';
  import { Icon } from '@immich/ui';
  import { mdiHistory } from '@mdi/js';
  import { RadioGroup } from 'bits-ui';
  import { earlierChoices, earlierFiles, earlierHint, earlierTitle } from './earlierExport';
  import type { EarlierExports, EarlierQuery } from './earlierExports.svelte';

  let {
    earlier,
    query,
    replaceReason = null,
    class: className = 'mt-1.5'
  }: {
    earlier: EarlierExports;
    query: EarlierQuery | null;
    replaceReason?: string | null;
    class?: string;
  } = $props();

  $effect(() => {
    earlier.watch(query);
  });

  $effect(() => () => earlier.watch(null));

  const titleId = $props.id();
  const run = $derived(earlier.run);
  const policy = $derived<OnConflict>(
    earlier.policy === 'replace' && replaceReason !== null ? 'keep_both' : earlier.policy
  );
</script>

{#if run}
  {@const files = earlierFiles(run.found.files)}
  <div
    transition:reveal
    class="{className} flex flex-col gap-2 rounded-lg border border-hairline p-2.5 touch:gap-2.5 touch:p-3"
  >
    <div class="flex items-start gap-2">
      <Icon
        icon={mdiHistory}
        size="16px"
        class="mt-px shrink-0 text-dark/60 touch:size-5"
        aria-hidden="true"
      />
      <p class="min-w-0 text-xs leading-snug text-dark touch:text-sm">
        <span id={titleId}>{earlierTitle(run)}</span>
        {#if files}
          <span class="block font-mono text-[11px] break-all text-dark/65 touch:text-xs">
            {files}
          </span>
        {/if}
      </p>
    </div>
    <RadioGroup.Root
      bind:value={() => policy, (v) => (earlier.policy = v as OnConflict)}
      orientation="horizontal"
      aria-labelledby={titleId}
      class={segmented.text.group}
    >
      {#each earlierChoices(run) as choice (choice.value)}
        <RadioGroup.Item
          value={choice.value}
          disabled={choice.value === 'replace' && replaceReason !== null}
          class="{segmented.text
            .radio} flex-1 px-1 disabled:pointer-events-none disabled:opacity-40"
        >
          {choice.label}
        </RadioGroup.Item>
      {/each}
    </RadioGroup.Root>
    <p class="text-[10px] leading-snug text-dark/60 touch:text-xs">
      {earlierHint(run, policy)}
      {#if replaceReason}
        <span class="block">{replaceReason}</span>
      {/if}
    </p>
  </div>
{/if}
