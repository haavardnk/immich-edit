<script lang="ts" generics="T extends string">
  import { chipRow, chipTone } from '#lib/components/chips/chipStyles.ts';
  import { Icon } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  type Option = { value: T; label: string; icon?: string };

  let {
    options,
    value,
    label,
    onChange
  }: {
    options: readonly Option[];
    value: T;
    label: string;
    onChange: (value: T) => void;
  } = $props();

  function pick(picked: string): void {
    const option = options.find((candidate) => candidate.value === picked);
    if (option) onChange(option.value);
  }
</script>

<RadioGroup.Root
  bind:value={() => value, pick}
  orientation="horizontal"
  aria-label={label}
  class={chipRow}
>
  {#each options as option (option.value)}
    <RadioGroup.Item value={option.value} class={chipTone.value}>
      {#if option.icon}
        <Icon icon={option.icon} size="1.15em" class="shrink-0" aria-hidden="true" />
      {/if}
      {option.label}
    </RadioGroup.Item>
  {/each}
</RadioGroup.Root>
