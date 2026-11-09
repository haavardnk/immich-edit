<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import type { ParamRange } from '#lib/editor/params/types.ts';
  import { valueEntry, type ValueEntryTarget } from '#lib/stores/valueEntry.svelte.ts';
  import { Button } from '@immich/ui';
  import { entryValue } from './sliderExpression';
  import { entryText } from './sliderStep';

  let {
    label,
    value,
    range,
    resetValue,
    modified,
    disabled,
    onLive,
    onCommit
  }: {
    label: string;
    value: number;
    range: ParamRange;
    resetValue: number;
    modified: boolean;
    disabled: boolean;
    onLive: (value: number) => void;
    onCommit: () => void;
  } = $props();

  const display = $derived(range.format(Object.is(value, -0) ? 0 : value));
  let typing = $state(false);
  let draft = $state('');
  let start = 0;
  let field = $state<HTMLInputElement | null>(null);
  let button = $state<HTMLButtonElement | null>(null);
  let target = $state.raw<ValueEntryTarget | null>(null);
  const editing = $derived(target !== null && valueEntry.target === target);

  $effect(() => {
    if (!typing || !field) return;
    field.focus();
    field.select();
  });

  $effect(() => () => {
    if (target) valueEntry.release(target);
  });

  function finish(commit: boolean): void {
    target = null;
    if (!commit) onLive(start);
    else if (value !== start) onCommit();
  }

  function openKeypad(owner: HTMLElement): void {
    target = {
      label,
      range,
      start,
      resetValue,
      owner,
      live: onLive,
      finish
    };
    valueEntry.open(target);
  }

  function open(): void {
    if (disabled) return;
    start = value;
    if (layout.mode === 'touch' && button) {
      openKeypad(button);
      return;
    }
    draft = entryText(value, range);
    typing = true;
  }

  function onDraftInput(e: Event): void {
    draft = (e.currentTarget as HTMLInputElement).value;
    const next = entryValue(draft, start, range);
    if (next !== null && next !== value) onLive(next);
  }

  function commitTyping(): void {
    if (!typing) return;
    typing = false;
    const next = entryValue(draft, start, range) ?? start;
    if (next !== value) onLive(next);
    if (next !== start) onCommit();
  }

  function cancelTyping(): void {
    typing = false;
    onLive(start);
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      commitTyping();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelTyping();
    }
  }
</script>

{#if typing}
  <TextInput
    bind:ref={field}
    inputmode="decimal"
    autocomplete="off"
    spellcheck="false"
    size="tiny"
    class="h-5 bg-hairline ring-1 ring-primary [&_input]:px-1 [&_input]:py-0 [&_input]:text-right [&_input]:font-mono [&_input]:text-[10px] [&_input]:tabular-nums"
    aria-label="{label} value"
    value={draft}
    oninput={onDraftInput}
    onblur={commitTyping}
    onkeydown={onKeyDown}
  />
{:else}
  <Button
    bind:ref={button}
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-5 min-h-0 w-full justify-end rounded px-1 py-0 text-right font-mono text-[10px] tabular-nums touch:h-8 touch:text-xs {editing
      ? 'bg-primary/10 text-primary outline-2 outline-primary'
      : modified
        ? 'bg-primary/10 text-primary'
        : 'text-dark/65'}"
    aria-label="Edit {label} value"
    onclick={open}
  >
    {display}
  </Button>
{/if}
