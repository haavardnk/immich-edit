<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import { Button } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    title,
    subtitle,
    submitLabel,
    saving,
    disabled,
    onSubmit,
    onClose,
    lead,
    children
  }: {
    title: string;
    subtitle?: string;
    submitLabel: string;
    saving: boolean;
    disabled: boolean;
    onSubmit: () => void;
    onClose: () => void;
    lead?: Snippet;
    children: Snippet;
  } = $props();

  const formId = $props.id();

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!disabled && !saving) onSubmit();
  }
</script>

<Dialog {title} {subtitle} {onClose}>
  {#snippet footer()}
    <div class="flex w-full items-center justify-end gap-2">
      {@render lead?.()}
      <Button
        type="button"
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:h-11"
        onclick={onClose}
      >
        Cancel
      </Button>
      <Button
        type="submit"
        form={formId}
        size="small"
        class="touch:h-11"
        loading={saving}
        disabled={disabled || saving}
      >
        {submitLabel}
      </Button>
    </div>
  {/snippet}

  <form id={formId} class="flex flex-col gap-4" onsubmit={submit}>
    {@render children()}
  </form>
</Dialog>
