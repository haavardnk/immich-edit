<script lang="ts">
  import TextInput from '#lib/components/TextInput.svelte';
  import {
    DEFAULT_FILENAME_TEMPLATE,
    FILENAME_TOKENS,
    insertToken,
    renderTemplate,
    templateError,
    type NameContext
  } from '#lib/editor/filenameTemplate.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { Button, Tooltip } from '@immich/ui';
  import { mergeProps } from 'bits-ui';
  import { tick } from 'svelte';
  import FilenamePreview from './FilenamePreview.svelte';

  let {
    value = $bindable(),
    example = null,
    extension
  }: {
    value: string;
    example?: NameContext | null;
    extension: string;
  } = $props();

  const id = $props.id();
  const touch = $derived(layout.mode === 'touch');
  const error = $derived(templateError(value));
  const preview = $derived(
    !error && example ? `${renderTemplate(value, example)}.${extension}` : null
  );

  let input = $state<HTMLInputElement | null>(null);
  let selection: { start: number | null; end: number | null } = { start: null, end: null };
  let keepFocus = false;

  function remember(): void {
    if (!input) return;
    selection = { start: input.selectionStart, end: input.selectionEnd };
  }

  function holdFocus(event: MouseEvent): void {
    keepFocus = document.activeElement === input;
    if (keepFocus) event.preventDefault();
  }

  async function insert(token: string): Promise<void> {
    if (keepFocus) remember();
    const edit = insertToken(value, token, selection.start, selection.end);
    value = edit.value;
    selection = { start: edit.caret, end: edit.caret };
    if (!keepFocus) return;
    keepFocus = false;
    await tick();
    input?.setSelectionRange(edit.caret, edit.caret);
  }
</script>

<div class="flex flex-col gap-1.5 touch:gap-2">
  <TextInput
    label="Filename"
    compact
    color="neutral"
    class="ring-0 focus-within:ring-1 focus-within:ring-primary"
    bind:ref={input}
    bind:value
    onblur={remember}
    placeholder={DEFAULT_FILENAME_TEMPLATE}
    aria-invalid={error ? 'true' : undefined}
    aria-describedby={error || preview ? `${id}-result` : undefined}
  />
  <div class="editor-compact-indent">
    <div
      role="group"
      aria-label="Filename tokens"
      class="flex flex-wrap gap-1 scrollbar-hidden touch:-mx-3 touch:flex-nowrap touch:gap-1.5 touch:overflow-x-auto touch:px-3"
    >
      {#each FILENAME_TOKENS as { token, label } (token)}
        {#if touch}
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            shape="round"
            class="h-9 shrink-0 gap-1.5 bg-ghost px-3 text-[13px] font-normal text-dark/70 hover:text-dark"
            aria-label="Insert {token}, {label}"
            onclick={() => insert(token)}
          >
            <code class="font-mono text-primary">{token}</code>{label}
          </Button>
        {:else}
          <Tooltip text={label}>
            {#snippet child({ props })}
              <Button
                size="tiny"
                variant="ghost"
                color="secondary"
                shape="round"
                class="h-6 shrink-0 bg-ghost px-1.5 font-mono text-[11px] font-normal text-primary hover:bg-primary/15"
                aria-label="Insert {token}, {label}"
                {...mergeProps(props, { onmousedown: holdFocus, onclick: () => insert(token) })}
              >
                {token}
              </Button>
            {/snippet}
          </Tooltip>
        {/if}
      {/each}
    </div>
    {#if error}
      <p
        id="{id}-result"
        class="rounded-lg bg-danger/10 px-2.5 py-1.5 text-xs leading-snug text-danger-800"
      >
        {error}
      </p>
    {:else if preview}
      <FilenamePreview id="{id}-result" name={preview} />
    {/if}
  </div>
</div>
