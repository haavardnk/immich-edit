<script lang="ts" generics="T extends string">
  import { RadioGroup } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import { segmented } from '#lib/components/segmented.ts';

  let {
    options,
    value,
    label,
    onChange,
    status,
    children
  }: {
    options: readonly { value: T; label: string }[];
    value: T;
    label: string;
    onChange: (value: T) => void;
    status?: string;
    children?: Snippet;
  } = $props();
</script>

<RadioGroup.Root
  {value}
  onValueChange={(picked) => {
    const option = options.find((candidate) => candidate.value === picked);
    if (option) onChange(option.value);
  }}
  orientation="horizontal"
  aria-label={label}
  class="{segmented.text.group} grid w-fit touch:h-12"
  style="grid-template-columns: repeat({options.length}, minmax(0, 1fr));"
>
  {#each options as option (option.value)}
    <RadioGroup.Item
      value={option.value}
      class="{segmented.text.radio} px-3 touch:h-11 touch:text-xs"
    >
      {option.label}
    </RadioGroup.Item>
  {/each}
</RadioGroup.Root>
{#if status}
  <p class="text-xs text-dark/65" aria-live="polite">{status}</p>
{/if}
{@render children?.()}
