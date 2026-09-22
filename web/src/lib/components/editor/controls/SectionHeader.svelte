<script lang="ts">
  import type { Snippet } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { DevelopSection } from '$lib/types/edits';
  import { Button } from '@immich/ui';
  import ResetButton from './ResetButton.svelte';

  let {
    title,
    onReset,
    modified = true,
    resetTitle = `Reset ${title}`,
    actions,
    section
  }: {
    title: string;
    onReset: (e: MouseEvent) => void;
    modified?: boolean;
    resetTitle?: string;
    actions?: Snippet;
    section?: DevelopSection;
  } = $props();

  let bypassing = $state(false);

  function startBypass(): void {
    if (bypassing || !section) return;
    bypassing = true;
    editor.bypassSection(section);
  }

  function endBypass(): void {
    if (!bypassing) return;
    bypassing = false;
    editor.endBypass();
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== ' ' && e.key !== 'Enter') return;
    e.preventDefault();
    startBypass();
  }

  function onKeyUp(e: KeyboardEvent): void {
    if (e.key !== ' ' && e.key !== 'Enter') return;
    endBypass();
  }
</script>

<div class="flex h-6 items-center justify-between border-b border-white/6">
  {#if section && modified}
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="-ms-1 h-5 min-h-0 rounded px-1 py-0 text-[9px] font-semibold uppercase select-none {bypassing
        ? 'text-dark/30'
        : 'text-dark/65'} hover:bg-hairline hover:text-dark"
      title="hold to bypass {title}"
      aria-label="Bypass {title}"
      onpointerdown={startBypass}
      onpointerup={endBypass}
      onpointerleave={endBypass}
      onpointercancel={endBypass}
      onkeydown={onKeyDown}
      onkeyup={onKeyUp}
      onblur={endBypass}
    >
      {title}
    </Button>
  {:else}
    <div class="text-[9px] font-semibold uppercase text-dark/65">{title}</div>
  {/if}
  <div class="flex items-center gap-0.5">
    {@render actions?.()}
    <ResetButton title={resetTitle} disabled={!modified} onclick={onReset} />
  </div>
</div>
