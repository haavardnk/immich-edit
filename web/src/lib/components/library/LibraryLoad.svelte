<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import { library, type LibraryView } from '#lib/stores/library.svelte.ts';
  import { Button, LoadingSpinner } from '@immich/ui';
  import type { Snippet } from 'svelte';

  let {
    view,
    label,
    children
  }: {
    view: LibraryView;
    label: string;
    children: Snippet;
  } = $props();

  const error = $derived(library.errorFor(view));
</script>

{#if library.isLoaded(view)}
  {@render children()}
{:else if error}
  <div class="flex justify-center px-4 py-6 sm:px-5">
    <Notice message={error} class="text-sm">
      <Button
        size="tiny"
        variant="outline"
        color="danger"
        class="ms-1 touch:min-h-11"
        onclick={() => library.request(view)}
      >
        Retry
      </Button>
    </Notice>
  </div>
{:else}
  <div class="flex items-center justify-center gap-2 px-4 py-6 text-muted sm:px-5" role="status">
    <LoadingSpinner size="small" />
    <span class="text-xs">Loading {label}…</span>
  </div>
{/if}
