<script lang="ts">
  import { correctionText, parseCorrection } from '#lib/metadata/clockText.ts';
  import TextInput from '#lib/components/TextInput.svelte';
  import { Button, IconButton } from '@immich/ui';
  import { mdiMinus, mdiPlus } from '@mdi/js';

  const MINUTE = 60_000;
  const HOUR = 60 * MINUTE;

  let {
    label,
    name,
    value,
    limit,
    onChange
  }: {
    label: string;
    name: string;
    value: number;
    limit?: number;
    onChange: (ms: number) => void;
  } = $props();

  let typed = $state<string | null>(null);

  function commit(): void {
    if (typed === null) return;
    const parsed = parseCorrection(typed, limit);
    if (parsed !== null) onChange(parsed);
    typed = null;
  }
</script>

<div class="flex items-center gap-3 touch:flex-col touch:items-stretch touch:gap-2">
  <div class="min-w-0 flex-1 text-sm text-light-950">{label}</div>
  <div
    class="flex w-60 shrink-0 items-center rounded-lg bg-light-100 ring-1 ring-hairline touch:w-full"
  >
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-8 px-2 whitespace-nowrap touch:h-11"
      aria-label="One hour earlier"
      onclick={() => onChange(value - HOUR)}
    >
      −1 h
    </Button>
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:size-11"
      icon={mdiMinus}
      title="One minute earlier"
      aria-label="One minute earlier"
      onclick={() => onChange(value - MINUTE)}
    />
    <TextInput
      compact
      class="w-full font-semibold tabular-nums [&_input]:h-8 [&_input]:py-0 [&_input]:text-center touch:[&_input]:h-9"
      aria-label={name}
      value={typed ?? correctionText(value)}
      oninput={(event) => (typed = event.currentTarget.value)}
      onblur={commit}
      onkeydown={(event) => {
        if (event.key === 'Enter') commit();
      }}
    />
    <IconButton
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:size-11"
      icon={mdiPlus}
      title="One minute later"
      aria-label="One minute later"
      onclick={() => onChange(value + MINUTE)}
    />
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="h-8 px-2 whitespace-nowrap touch:h-11"
      aria-label="One hour later"
      onclick={() => onChange(value + HOUR)}
    >
      +1 h
    </Button>
  </div>
</div>
