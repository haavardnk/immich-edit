<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiDotsHorizontal } from '@mdi/js';
  import type { Snippet } from 'svelte';

  let {
    label,
    title,
    subtitle,
    size = 'small',
    class: className = '',
    children
  }: {
    label: string;
    title: string;
    subtitle?: string;
    size?: 'tiny' | 'small';
    class?: string;
    children: Snippet<[() => void]>;
  } = $props();

  let open = $state(false);
</script>

<Popover
  {open}
  onOpenChange={(next) => (open = next)}
  tooltip={label}
  {title}
  {subtitle}
  contentClass="w-52 py-1"
>
  {#snippet trigger(props)}
    <IconButton
      {...props}
      size={layout.mode === 'touch' ? 'medium' : size}
      variant="ghost"
      color="secondary"
      shape="round"
      class="touch:size-11 {className}"
      icon={mdiDotsHorizontal}
      title=""
      aria-label={label}
    />
  {/snippet}
  {@render children(() => (open = false))}
</Popover>
