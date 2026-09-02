<script lang="ts">
  import TextInput from './TextInput.svelte';

  let {
    value = $bindable(''),
    ariaLabel,
    placeholder,
    compact = false,
    round = false,
    commitOnBlur = false,
    oncommit,
    oncancel
  }: {
    value?: string;
    ariaLabel: string;
    placeholder?: string;
    compact?: boolean;
    round?: boolean;
    commitOnBlur?: boolean;
    oncommit: () => void | Promise<void>;
    oncancel: () => void;
  } = $props();

  let input = $state<HTMLInputElement | null>(null);
  let canceled = false;

  $effect(() => {
    input?.focus();
    input?.select();
  });

  function onKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' && event.key !== 'Escape') return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === 'Escape') {
      canceled = true;
      oncancel();
      return;
    }
    if (commitOnBlur) {
      input?.blur();
      return;
    }
    void oncommit();
  }

  function onBlur(): void {
    if (commitOnBlur && !canceled) void oncommit();
  }
</script>

<TextInput
  size={compact ? undefined : 'tiny'}
  {compact}
  shape={round ? 'round' : undefined}
  {placeholder}
  aria-label={ariaLabel}
  bind:value
  bind:ref={input}
  onkeydown={onKeydown}
  onblur={onBlur}
  onclick={(event) => event.stopPropagation()}
/>
