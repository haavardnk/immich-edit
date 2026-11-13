<script lang="ts">
  import { RETOUCH_MODE_ICON } from '#lib/editor/retouch/modes.ts';
  import { NO_STROKES_TEXT, strokeName } from '#lib/editor/retouch/text.ts';
  import { scrollSideways } from '#lib/input/sidewaysWheel.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { revealAcross } from '#lib/utils/scrollReveal.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiClose, mdiDotsHorizontal, mdiEyeOffOutline, mdiEyeOutline } from '@mdi/js';

  const REVEAL_MARGIN = 16;

  let { onmenu }: { onmenu?: () => void } = $props();

  let strip = $state<HTMLElement | null>(null);
  const strokes = $derived(editor.edits.retouch);
  const count = $derived(strokes.length);

  $effect(() => {
    if (strip) strip.scrollLeft = strip.scrollWidth;
  });

  $effect(() => {
    const id = editor.activeRetouchId;
    void count;
    if (!strip || !id) return;
    const chip = strip.querySelector<HTMLElement>(`[data-stroke-id="${id}"]`);
    if (chip) revealAcross(chip, strip, REVEAL_MARGIN);
  });
</script>

<div class="-mx-4 flex shrink-0 items-center {onmenu ? 'pe-3' : ''}">
  {#if count > 0}
    <div
      bind:this={strip}
      data-testid="retouch-strokes"
      class="flex min-w-0 flex-1 gap-2 overflow-x-auto overscroll-x-contain px-4 py-1 scrollbar-hidden fade-x"
      onwheel={scrollSideways}
    >
      {#each strokes as stroke, i (stroke.id)}
        {@const name = strokeName(i)}
        {@const active = editor.activeRetouchId === stroke.id}
        <div
          data-stroke-id={stroke.id}
          class="flex h-12 shrink-0 items-center rounded-xl border pe-1 transition-colors {active
            ? 'border-primary bg-primary/10'
            : 'border-transparent bg-ghost'}"
        >
          <Button
            type="button"
            size="medium"
            variant="ghost"
            color="secondary"
            class="h-full gap-2 rounded-xl ps-3 pe-1.5 text-[14px] font-normal text-dark hover:bg-transparent {stroke.enabled
              ? ''
              : 'opacity-50'}"
            leadingIcon={RETOUCH_MODE_ICON[stroke.mode]}
            aria-pressed={active}
            onclick={() => (editor.activeRetouchId = stroke.id)}
          >
            {name}
          </Button>
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            class="size-10"
            icon={stroke.enabled ? mdiEyeOutline : mdiEyeOffOutline}
            aria-label={stroke.enabled ? `Hide ${name}` : `Show ${name}`}
            onclick={() => void editor.toggleRetouchStroke(stroke.id)}
          />
          <IconButton
            size="medium"
            variant="ghost"
            color="secondary"
            class="size-10"
            icon={mdiClose}
            aria-label="Delete {name}"
            onclick={() => void editor.removeRetouchStroke(stroke.id)}
          />
        </div>
      {/each}
    </div>
  {:else}
    <p class="flex h-14 min-w-0 flex-1 items-center px-4 text-[13px] text-dark/65">
      {NO_STROKES_TEXT}
    </p>
  {/if}
  {#if onmenu}
    <IconButton
      size="medium"
      variant="ghost"
      color="secondary"
      class="my-1 size-12 shrink-0 rounded-xl bg-ghost"
      icon={mdiDotsHorizontal}
      aria-label="Retouch actions"
      onclick={onmenu}
    />
  {/if}
</div>
