<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { SEARCH_FIELD_SIZES, type SearchFieldSize } from '#lib/components/searchFieldSize.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiCloseCircle, mdiMagnify } from '@mdi/js';

  let {
    value = $bindable(''),
    ref = $bindable(null),
    label,
    placeholder = label,
    size = 'default',
    oninput,
    onkeydown
  }: {
    value?: string;
    ref?: HTMLInputElement | null;
    label: string;
    placeholder?: string;
    size?: SearchFieldSize;
    oninput?: (value: string) => void;
    onkeydown?: (event: KeyboardEvent) => void;
  } = $props();

  const inputSize = $derived(SEARCH_FIELD_SIZES[size]);

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
{/snippet}

<TextInput
  bind:ref
  {value}
  type="text"
  size="tiny"
  color="neutral"
  autocomplete="off"
  {placeholder}
  aria-label={label}
  leadingIcon={searchIcon}
  trailingIcon={value ? clearButton : undefined}
  class="rounded-lg {inputSize}"
  oninput={(event: Event) => set((event.currentTarget as HTMLInputElement).value)}
  onkeydown={keydown}
/>
