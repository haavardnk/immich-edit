<script lang="ts">
  import { modifiedTabCount } from '#lib/editor/modified.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { touchUi } from '#lib/stores/touchUi.svelte.ts';
  import { Icon, IconButton } from '@immich/ui';
  import { mdiFilmstripBox } from '@mdi/js';
  import { Tabs } from 'bits-ui';
  import { openTouchMode, retapTouchMode, touchMode, TOUCH_MODES, type TouchMode } from './modes';

  let { rail = false, filmstrip = false }: { rail?: boolean; filmstrip?: boolean } = $props();
</script>

<Tabs.Root
  value={touchMode()}
  onValueChange={(value) => openTouchMode(value as TouchMode)}
  activationMode="manual"
  orientation={rail ? 'vertical' : 'horizontal'}
  class={rail
    ? 'flex w-18 shrink-0 flex-col items-center border-l border-hairline bg-editor-chrome py-2'
    : 'shrink-0 border-t border-hairline bg-editor-chrome pb-(--tab-bar-pad)'}
>
  <Tabs.List
    aria-label="Editor modes"
    class={rail ? 'flex flex-col items-center gap-1' : 'flex justify-around'}
  >
    {#each TOUCH_MODES as mode (mode.id)}
      {@const modified = mode.id !== 'presets' && modifiedTabCount(editor.edits, mode.id) > 0}
      <Tabs.Trigger
        value={mode.id}
        onclick={(event) => retapTouchMode(event, mode.id)}
        class="group flex min-w-0 items-center justify-center outline-none {rail
          ? 'h-16 w-16'
          : 'h-14 flex-1'}"
      >
        <span
          class="relative flex h-12 w-17 max-w-full flex-col items-center justify-center gap-0.5 rounded-xl text-[11px] font-medium group-focus-visible:ring-2 group-focus-visible:ring-primary group-data-[state=active]:bg-primary/12 group-data-[state=active]:text-light-950 group-data-[state=inactive]:text-dark/65"
        >
          <Icon
            icon={mode.icon}
            size="22px"
            class="group-data-[state=active]:text-primary"
            aria-hidden="true"
          />
          {mode.label}
          {#if modified}
            <span
              data-modified-indicator
              aria-hidden="true"
              class="absolute top-1 right-3 size-1.5 rounded-full bg-primary"
            ></span>
          {/if}
        </span>
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
  {#if filmstrip}
    <IconButton
      size="large"
      variant="ghost"
      color={touchUi.filmstrip ? 'primary' : 'secondary'}
      class="mt-auto"
      icon={mdiFilmstripBox}
      title={touchUi.filmstrip ? 'Hide filmstrip' : 'Show filmstrip'}
      aria-label="Filmstrip"
      aria-pressed={touchUi.filmstrip}
      onclick={touchUi.toggleFilmstrip}
    />
  {/if}
</Tabs.Root>
