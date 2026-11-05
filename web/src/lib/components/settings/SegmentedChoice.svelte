<script lang="ts" generics="T extends string">
  import { Icon, Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import { segmented } from '#lib/components/segmented.ts';

  type Option = { value: T; label: string; icon?: string; title?: string };

  let {
    options,
    value,
    label,
    onChange,
    status,
    fill = false,
    children
  }: {
    options: readonly Option[];
    value: T;
    label: string;
    onChange: (value: T) => void;
    status?: string;
    fill?: boolean;
    children?: Snippet;
  } = $props();
</script>

{#snippet item(option: Option, props: Record<string, unknown>)}
  <RadioGroup.Item
    {...props}
    value={option.value}
    class="{segmented.text.radio} gap-1.5 px-3 touch:h-11 touch:text-xs"
  >
    {#if option.icon}
      <Icon icon={option.icon} size="16px" class="shrink-0" aria-hidden="true" />
    {/if}
    {option.label}
  </RadioGroup.Item>
{/snippet}

<RadioGroup.Root
  {value}
  onValueChange={(picked) => {
    const option = options.find((candidate) => candidate.value === picked);
    if (option) onChange(option.value);
  }}
  orientation="horizontal"
  aria-label={label}
  class="{segmented.text.group} grid touch:h-12 {fill ? 'w-full' : 'w-fit'}"
  style="grid-template-columns: repeat({options.length}, minmax(0, 1fr));"
>
  {#each options as option (option.value)}
    {#if option.title}
      <Tooltip text={option.title}>
        {#snippet child({ props })}
          {@render item(option, props)}
        {/snippet}
      </Tooltip>
    {:else}
      {@render item(option, {})}
    {/if}
  {/each}
</RadioGroup.Root>
{#if status}
  <p class="text-xs text-dark/65" aria-live="polite">{status}</p>
{/if}
{@render children?.()}
