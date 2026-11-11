<script lang="ts">
  import { formatChord } from '#lib/shortcuts/chord.ts';
  import { isCustomisable, isCustomised, keysOf, resetKey } from '#lib/shortcuts/custom.ts';
  import { defaultKeysFor, keysFor } from '#lib/shortcuts/labels.ts';
  import type { Keybind, KeybindId } from '#lib/shortcuts/table.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import { Button, IconButton, Kbd } from '@immich/ui';
  import { mdiRestore } from '@mdi/js';
  import ConflictPanel from './ConflictPanel.svelte';
  import type { ShortcutRecorder } from './recorder.svelte.ts';

  let { bind, recorder }: { bind: Keybind; recorder: ShortcutRecorder } = $props();

  const NOTICES = {
    unsupported: 'That key cannot be used; try a letter, digit or symbol',
    browser: 'The browser keeps that shortcut; try another'
  } as const;

  const editable = $derived(isCustomisable(bind));
  const custom = $derived(isCustomised(bind.id, preferences.shortcuts));
  const keys = $derived(keysFor(bind.id as KeybindId));
  const recording = $derived(recorder.id === bind.id);
  const gives = $derived(
    keysOf(bind.id, preferences.shortcuts)
      .map((spec) => formatChord(spec))
      .join(' / ')
  );

  function leave(): void {
    if (recording && !recorder.pending) recorder.cancel();
  }

  function reset(): void {
    void preferences.setShortcuts(resetKey(preferences.shortcuts, bind.id));
  }
</script>

{#if editable}
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-auto justify-self-start rounded-md p-0 hover:bg-transparent"
    aria-label="Change the shortcut for {bind.label}"
    data-shortcut={bind.id}
    onclick={() => recorder.start(bind.id)}
    onblur={leave}
  >
    <Kbd
      size="tiny"
      class="whitespace-nowrap {recording
        ? 'border-primary text-primary ring-2 ring-primary/40'
        : custom
          ? 'border-primary/60 bg-primary/10 text-primary'
          : 'text-dark/90 hover:border-dark/40'}"
    >
      {recording ? 'Press keys…' : keys || 'None'}
    </Kbd>
  </Button>
{:else}
  <span class="font-mono whitespace-nowrap text-dark/90">{keys}</span>
{/if}
<span class="text-dark/70">
  {bind.label}
  {#if custom}
    <span class="text-dark/50">· default {defaultKeysFor(bind.id)}</span>
  {/if}
</span>
<span class="flex size-6 items-center justify-center">
  {#if custom}
    <IconButton
      size="tiny"
      variant="ghost"
      color="secondary"
      icon={mdiRestore}
      title="Reset to {defaultKeysFor(bind.id)}"
      aria-label="Reset the shortcut for {bind.label}"
      onclick={reset}
    />
  {/if}
</span>
{#if recording && recorder.pending}
  <ConflictPanel
    spec={recorder.pending.spec}
    conflict={recorder.pending.conflict}
    {gives}
    onswap={recorder.swap}
    oncancel={recorder.cancel}
  />
{:else if recording && recorder.notice}
  <p role="status" class="col-span-3 mb-1 text-warning-800">{NOTICES[recorder.notice]}</p>
{/if}
