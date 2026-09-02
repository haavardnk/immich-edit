<script lang="ts">
  import TextInput from '$lib/components/TextInput.svelte';
  import { IconButton } from '@immich/ui';
  import { mdiClose, mdiMagnify } from '@mdi/js';

  let {
    value,
    onInput,
    onSubmit,
    onClear
  }: {
    value: string;
    onInput: (event: Event) => void;
    onSubmit: (event: SubmitEvent) => void;
    onClear: () => void;
  } = $props();
</script>

<form
  role="search"
  class="relative w-full max-w-4xl text-sm select-text"
  autocomplete="off"
  onsubmit={onSubmit}
>
  <TextInput
    id="main-search-bar"
    name="q"
    type="text"
    class="h-11.5 rounded-xl bg-ghost text-white/80 ring-transparent transition-colors hover:bg-white/7 dark:bg-ghost dark:ring-transparent dark:hover:bg-white/7 [&_input]:py-0 [&_input]:ps-13 [&_input]:placeholder:text-white/35 {value
      ? 'pe-22.5'
      : 'pe-14'}"
    placeholder="Search your photos"
    aria-label="Search your photos"
    {value}
    oninput={onInput}
  />
  {#if value}
    <div class="absolute inset-y-0 inset-e-0 flex items-center pe-2">
      <IconButton
        type="button"
        size="medium"
        variant="ghost"
        color="secondary"
        shape="round"
        icon={mdiClose}
        title="Clear search"
        aria-label="Clear search"
        onclick={onClear}
      />
    </div>
  {/if}
  <div class="absolute inset-y-0 inset-s-0 flex items-center ps-2">
    <IconButton
      type="submit"
      size="medium"
      variant="ghost"
      color="secondary"
      shape="round"
      icon={mdiMagnify}
      title="Search"
      aria-label="Search"
    />
  </div>
</form>
