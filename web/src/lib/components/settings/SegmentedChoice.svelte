<script lang="ts" generics="T extends string">
  import { Icon, Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import { segmented } from '#lib/components/segmented.ts';

  type Option = { value: T; label: string; icon?: string; title?: string; disabled?: boolean };

  const LOOKS = {
    panel: {
      group: `${segmented.text.group} grid touch:h-12`,
      item: `${segmented.text.radio} gap-1.5 px-3 touch:h-11 touch:text-xs`,
      icon: 'shrink-0'
    },
    photo: {
      group: segmented.photo.group,
      item: segmented.photo.radio,
      icon: segmented.photo.icon
    }
  } as const;

  let {
    options,
    value,
    label,
    onChange,
    status,
    fill = false,
    appearance = 'panel',
    children
  }: {
    options: readonly Option[];
    value: T;
    label: string;
    onChange: (value: T) => void;
    status?: string;
    fill?: boolean;
    appearance?: keyof typeof LOOKS;
    children?: Snippet;
  } = $props();

  const look = $derived(LOOKS[appearance]);

  function pick(picked: string): void {
    const option = options.find((candidate) => candidate.value === picked);
    if (option) onChange(option.value);
  }
</script>

{#snippet item(option: Option, props: Record<string, unknown>)}
  <RadioGroup.Item {...props} value={option.value} disabled={option.disabled} class={look.item}>
    {#if option.icon}
      <Icon icon={option.icon} size="16px" class={look.icon} aria-hidden="true" />
    {/if}
    {option.label}
  </RadioGroup.Item>
{/snippet}

<RadioGroup.Root
  bind:value={() => value, pick}
  orientation="horizontal"
  aria-label={label}
  class="{look.group} {fill ? 'w-full' : 'w-fit'}"
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
