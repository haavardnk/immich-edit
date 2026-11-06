<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { savedViewNameError } from '#lib/browse/savedViewName.ts';
  import { Button } from '@immich/ui';
  import { onMount, untrack } from 'svelte';

  let {
    initial,
    label,
    onSubmit,
    onCancel,
    class: className = ''
  }: {
    initial: string;
    label: string;
    onSubmit: (name: string) => Promise<unknown>;
    onCancel: () => void;
    class?: string;
  } = $props();

  let name = $state(untrack(() => initial));
  let input = $state<HTMLInputElement | null>(null);
  let busy = $state(false);

  const errorId = $props.id();
  const error = $derived(savedViewNameError(name));

  onMount(() => {
    requestAnimationFrame(() => {
      input?.focus();
      input?.select();
    });
  });

  async function submit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (error || busy) return;
    busy = true;
    try {
      await onSubmit(name.trim());
    } finally {
      busy = false;
    }
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key !== 'Escape') return;
    event.preventDefault();
    event.stopPropagation();
    onCancel();
  }
</script>

<form
  aria-label={label}
  class="flex flex-col gap-2 {className}"
  onsubmit={(event) => void submit(event)}
>
  <TextInput
    bind:ref={input}
    bind:value={name}
    type="text"
    size="tiny"
    autocomplete="off"
    aria-label={label}
    aria-invalid={error !== null}
    aria-describedby={error ? errorId : undefined}
    {onkeydown}
    class="rounded-lg bg-light-100! ring-1 ring-primary/20! focus-within:bg-light-100! focus-within:ring-primary! [&_input]:h-8 [&_input]:text-[13px] touch:[&_input]:h-11 touch:[&_input]:text-[15px]"
  />
  {#if error}
    <p id={errorId} class="text-[11.5px] text-danger-800 touch:text-[13px]">{error}</p>
  {/if}
  <div class="flex justify-end gap-2">
    <Button
      type="button"
      size="small"
      variant="ghost"
      color="secondary"
      class="touch:h-11 touch:px-4"
      onclick={onCancel}
    >
      Cancel
    </Button>
    <Button
      type="submit"
      size="small"
      class="touch:h-11 touch:px-4"
      disabled={error !== null || busy}
    >
      Save
    </Button>
  </div>
</form>
