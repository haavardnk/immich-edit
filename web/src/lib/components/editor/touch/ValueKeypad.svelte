<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import { entryValue } from '#lib/components/editor/controls/sliderExpression.ts';
  import { entryText, nudgeValue } from '#lib/components/editor/controls/sliderStep.ts';
  import { isPinnable } from '#lib/editor/quickSettings.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { panRecognizer } from '#lib/gestures/pan.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { valueEntry, type ValueEntryTarget } from '#lib/stores/valueEntry.svelte.ts';
  import { revealWithin, scrollParent } from '#lib/utils/scrollReveal.ts';
  import { Button, IconButton } from '@immich/ui';
  import {
    mdiBackspaceOutline,
    mdiMinus,
    mdiPinOffOutline,
    mdiPinOutline,
    mdiPlus,
    mdiPlusMinus
  } from '@mdi/js';
  import { tick, untrack } from 'svelte';
  import { pressKey, type KeypadKey, type KeypadText } from './keypadText';

  const SWIPE_CANCEL = 48;
  const REVEAL_MARGIN = 8;
  const DIGIT_ROWS: readonly (readonly KeypadKey[])[] = [
    ['1', '2', '3'],
    ['4', '5', '6'],
    ['7', '8', '9']
  ];

  let { target }: { target: ValueEntryTarget } = $props();

  let entry = $state<KeypadText>(
    untrack(() => ({ text: entryText(target.start, target.range), fresh: true }))
  );
  let hardware = $state(false);
  let field = $state<HTMLInputElement | null>(null);
  let pad = $state<HTMLElement | null>(null);
  let shown = untrack(() => target.start);
  const { label, range } = $derived(target);
  const setting = $derived(target.setting && isPinnable(target.setting) ? target.setting : null);
  const pinned = $derived(setting !== null && preferences.isPinned(setting));
  const hint = $derived(`${range.format(range.min)} … ${range.format(range.max)}`);

  $effect(() => {
    if (field) untrack(place);
  });

  $effect(() => {
    const owner = target.owner;
    const scroller = scrollParent(owner);
    if (!scroller) return;
    const observer = new ResizeObserver(() => revealWithin(owner, scroller, REVEAL_MARGIN));
    observer.observe(scroller);
    return () => observer.disconnect();
  });

  $effect(() => {
    const outside = (e: PointerEvent): void => {
      if (!(e.target instanceof Node)) return;
      if (pad?.contains(e.target) || target.owner.contains(e.target)) return;
      valueEntry.close(true);
    };
    window.addEventListener('pointerdown', outside, { capture: true });
    return () => window.removeEventListener('pointerdown', outside, { capture: true });
  });

  function place(): void {
    if (!field) return;
    field.focus({ preventScroll: true });
    if (entry.fresh) field.select();
    else field.setSelectionRange(field.value.length, field.value.length);
  }

  function render(text: string): void {
    const value = entryValue(text, target.start, range);
    if (value === null || value === shown) return;
    shown = value;
    target.live(value);
  }

  function show(next: KeypadText): void {
    entry = next;
    render(next.text);
    void tick().then(place);
  }

  function press(key: KeypadKey): void {
    show(pressKey(entry, key));
  }

  function nudge(steps: number): void {
    const next = nudgeValue(shown, range, steps, 'fine');
    show({ text: entryText(next, range), fresh: true });
  }

  function keep(e: PointerEvent): void {
    e.preventDefault();
  }

  function onFieldInput(e: Event): void {
    const text = (e.currentTarget as HTMLInputElement).value;
    entry = { text, fresh: false };
    render(text);
  }

  function onFieldKey(e: KeyboardEvent): void {
    if (e.key === 'Enter') {
      e.preventDefault();
      valueEntry.close(true);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      valueEntry.close(false);
    } else {
      hardware = true;
    }
  }

  const swipe = gestures({
    recognizers: [
      panRecognizer({
        lockAxis: true,
        end: (pan) => {
          if (pan.axis === 'y' && pan.dy > SWIPE_CANCEL) valueEntry.close(false);
        }
      })
    ]
  });
</script>

{#snippet key(name: KeypadKey)}
  <Button
    size="large"
    variant="ghost"
    color="secondary"
    class="h-11 rounded-lg bg-light-200 text-xl font-normal text-light-950 hover:bg-light-300"
    onpointerdown={keep}
    onclick={() => press(name)}
  >
    {name}
  </Button>
{/snippet}

<section
  bind:this={pad}
  aria-label="{label} keypad"
  class="shrink-0 border-t border-hairline bg-editor-chrome px-3 pt-2 pb-(--tab-bar-pad)"
  {@attach swipe}
>
  <div class="flex items-center gap-2 pb-2">
    {#if setting}
      {@const id = setting}
      <IconButton
        size="large"
        variant="ghost"
        color={pinned ? 'primary' : 'secondary'}
        class="size-10 shrink-0 rounded-[10px] {pinned ? 'bg-primary/15' : 'bg-light-200'}"
        icon={pinned ? mdiPinOffOutline : mdiPinOutline}
        title={pinned ? `Unpin ${label}` : `Pin ${label} to Quick`}
        aria-label={pinned ? `Unpin ${label}` : `Pin ${label} to Quick`}
        onpointerdown={keep}
        onclick={() => void preferences.togglePin(id)}
      />
    {/if}
    <IconButton
      size="large"
      variant="ghost"
      color="secondary"
      class="size-10 shrink-0 rounded-[10px] bg-light-200"
      icon={mdiMinus}
      title="Decrease {label}"
      aria-label="Decrease {label}"
      onpointerdown={keep}
      onclick={() => nudge(-1)}
    />
    <TextInput
      bind:ref={field}
      inputmode="none"
      autocomplete="off"
      spellcheck="false"
      class="h-10 flex-1 rounded-[10px] bg-light-100! outline-2 outline-primary [&_input]:px-3 [&_input]:selection:bg-primary/30 [&_input]:font-mono [&_input]:text-base [&_input]:tabular-nums"
      aria-label="{label} value"
      value={entry.text}
      oninput={onFieldInput}
      onkeydown={onFieldKey}
    />
    <IconButton
      size="large"
      variant="ghost"
      color="secondary"
      class="size-10 shrink-0 rounded-[10px] bg-light-200"
      icon={mdiPlus}
      title="Increase {label}"
      aria-label="Increase {label}"
      onpointerdown={keep}
      onclick={() => nudge(1)}
    />
  </div>
  {#if !hardware}
    <div class="grid grid-cols-4 gap-1.5 pb-2">
      {#each DIGIT_ROWS as digits, index (index)}
        {#each digits as digit (digit)}
          {@render key(digit)}
        {/each}
        {#if index === 0}
          <IconButton
            size="large"
            variant="ghost"
            color="secondary"
            class="h-11 w-full rounded-lg"
            icon={mdiBackspaceOutline}
            title="Delete"
            aria-label="Delete"
            onpointerdown={keep}
            onclick={() => press('back')}
          />
        {:else if index === 1}
          <IconButton
            size="large"
            variant="ghost"
            color="secondary"
            class="h-11 w-full rounded-lg"
            icon={mdiPlusMinus}
            title="Toggle sign"
            aria-label="Toggle sign"
            onpointerdown={keep}
            onclick={() => press('sign')}
          />
        {:else}
          <Button
            size="large"
            variant="ghost"
            color="secondary"
            class="h-11 rounded-lg text-xl font-normal"
            aria-label="Decimal point"
            onpointerdown={keep}
            onclick={() => press('.')}
          >
            .
          </Button>
        {/if}
      {/each}
      <Button
        size="large"
        variant="ghost"
        color="secondary"
        class="h-11 rounded-lg text-sm font-medium"
        onpointerdown={keep}
        onclick={() => show({ text: entryText(target.resetValue, range), fresh: true })}
      >
        Reset
      </Button>
      {@render key('0')}
      <span
        class="flex items-center justify-center text-center text-[11px] leading-tight text-dark/65 tabular-nums"
      >
        {hint}
      </span>
      <Button
        size="large"
        color="primary"
        class="h-11 rounded-lg text-sm font-medium"
        onpointerdown={keep}
        onclick={() => valueEntry.close(true)}
      >
        Done
      </Button>
    </div>
  {/if}
</section>
