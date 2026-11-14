<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiCloseCircle, mdiMagnify } from '@mdi/js';

  let {
    value = $bindable(''),
    ref = $bindable(null),
    label,
    placeholder = label,
    compact = false,
    oninput,
    onkeydown
  }: {
    value?: string;
    ref?: HTMLInputElement | null;
    label: string;
    placeholder?: string;
    compact?: boolean;
    oninput?: (value: string) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  } = $props();

  const inputSize = $derived(
    compact
      ? '[&_input]:h-6 [&_input]:text-[11px]'
      : '[&_input]:h-7.5 [&_input]:text-[13px] touch:[&_input]:h-11 touch:[&_input]:text-[15px]'
  );

  function set(next: string): void {
    value = next;
    oninput?.(next);
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      if (value) set('');
      else ref?.blur();
      return;
    }
    onkeydown?.(event);
  }
</script>

{#snippet searchIcon(_disabled: boolean)}
  <Icon icon={mdiMagnify} size="60%" class="text-dark/65" aria-hidden />
{/snippet}

{#snippet clearButton(disabled: boolean)}
  {#if value}
    <IconButton
      size="tiny"
      shape="round"
      variant="ghost"
      color="secondary"
      class="me-1"
      icon={mdiCloseCircle}
      title="Clear"
      aria-label="Clear {label.toLowerCase()}"
      {disabled}
      onclick={() => {
        set('');
        ref?.focus();
      }}
    />
  {/if}
{/snippet}

<TextInput
  bind:ref
  {value}
  type="text"
  size="tiny"
  autocomplete="off"
  {placeholder}
  aria-label={label}
  leadingIcon={searchIcon}
  trailingIcon={clearButton}
  class="rounded-lg bg-light-100! ring-1 ring-primary/20! focus-within:bg-light-100! focus-within:ring-primary! {inputSize}"
  oninput={(event: Event) => set((event.currentTarget as HTMLInputElement).value)}
  onkeydown={keydown}
/>
