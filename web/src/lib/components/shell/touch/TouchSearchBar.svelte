<script lang="ts">
  import { page } from '$app/state';
  import GlobalSearch from '#lib/components/GlobalSearch.svelte';
  import { SearchInput } from '#lib/browse/searchInput.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { Button } from '@immich/ui';
  import type { Attachment } from 'svelte/attachments';
  import { untrack } from 'svelte';

  const search = new SearchInput('/search');

  let field: HTMLElement | null = null;

  const bindField: Attachment<HTMLElement> = (element) => {
    field = element;
    if (!untrack(() => page.url.searchParams.get('q'))) element.querySelector('input')?.focus();
    return () => {
      if (field === element) field = null;
    };
  };

  function blur(): void {
    field?.querySelector('input')?.blur();
  }

  function onSubmit(event: SubmitEvent): void {
    search.submitEvent(event);
    blur();
  }

  function cancel(): void {
    search.clear();
    blur();
  }
</script>

<div class="flex min-w-0 flex-1 items-center" {@attach bindField}>
  <GlobalSearch
    value={ui.searchQuery}
    onInput={search.inputEvent}
    {onSubmit}
    onClear={search.clear}
    onFocus={search.focus}
  />
</div>
<Button
  size="small"
  variant="ghost"
  color="primary"
  class="min-h-11 shrink-0 px-2.5"
  onclick={cancel}
>
  Cancel
</Button>
