<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    confirmLabel,
    color = 'primary',
    onConfirm,
    onCancel,
    children
  }: {
    title: string;
    confirmLabel: string;
    color?: 'primary' | 'danger';
    onConfirm: () => void;
    onCancel: () => void;
    children: Snippet;
  } = $props();

  let confirmButton = $state<HTMLElement | null>(null);

  $effect(() => confirmButton?.focus());
</script>

<Dialog {title} onClose={onCancel}>
  {#snippet footer()}
    <div class="flex w-full justify-end gap-2">
      <Button size="tiny" variant="ghost" color="secondary" onclick={onCancel}>Cancel</Button>
      <Button bind:ref={confirmButton} size="tiny" {color} onclick={onConfirm}>
        {confirmLabel}
      </Button>
    </div>
  {/snippet}

  <div class="flex flex-col gap-2 text-xs text-dark/80">
    {@render children()}
  </div>
</Dialog>
