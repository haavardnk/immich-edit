<script lang="ts">
  import { formatChord } from '#lib/shortcuts/chord.ts';
  import type { Conflict } from '#lib/shortcuts/custom.ts';
  import { isCustomisable } from '#lib/shortcuts/custom.ts';
  import { CONTEXT_NAMES } from '#lib/shortcuts/stacks.ts';
  import { Button, Icon, Kbd } from '@immich/ui';
  import { mdiAlertOutline } from '@mdi/js';

  let {
    spec,
    conflict,
    gives,
    onswap,
    oncancel
  }: {
    spec: string;
    conflict: Conflict;
    gives: string;
    onswap: () => void;
    oncancel: () => void;
  } = $props();

  const swappable = $derived(isCustomisable(conflict.bind));
</script>

<div
  role="alert"
  class="col-span-3 mb-1 flex flex-col gap-2 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2.5"
>
  <p class="flex flex-wrap items-center gap-1.5 text-dark/80">
    <Icon icon={mdiAlertOutline} size="15px" class="shrink-0 text-warning" aria-hidden="true" />
    <Kbd size="tiny">{formatChord(spec)}</Kbd>
    already runs
    <strong class="font-semibold text-light-950">{conflict.bind.label}</strong>
    ({CONTEXT_NAMES[conflict.context]}).
  </p>
  <div class="flex flex-wrap items-center gap-2">
    {#if swappable}
      <Button size="tiny" shape="round" color="primary" onclick={onswap}>Swap</Button>
    {/if}
    <Button
      size="tiny"
      shape="round"
      variant="ghost"
      color="secondary"
      class="bg-light-200 hover:bg-light-300"
      onclick={oncancel}
    >
      Cancel
    </Button>
    {#if swappable}
      <span class="text-dark/65">
        {#if gives}
          Swap gives that action <Kbd size="tiny">{gives}</Kbd>
        {:else}
          Swap leaves that action without a key
        {/if}
      </span>
    {/if}
  </div>
</div>
