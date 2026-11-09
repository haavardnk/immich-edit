<script lang="ts">
  import { page } from '$app/state';
  import { backParent } from '#lib/components/shell/backParent.ts';
  import { crossesTabs } from '#lib/components/shell/touch/navTabs.ts';
  import { navHistory } from '#lib/stores/navHistory.ts';
  import { Button, IconButton } from '@immich/ui';
  import { mdiArrowLeft } from '@mdi/js';

  let { labelled = false, onBack }: { labelled?: boolean; onBack?: () => void } = $props();

  function back(): void {
    if (onBack) onBack();
    else
      navHistory.back(
        backParent(page.url),
        (previous) => !crossesTabs(new URL(previous, page.url.origin).pathname, page.url.pathname)
      );
  }
</script>

{#if labelled}
  <Button
    size="small"
    variant="outline"
    color="secondary"
    class="min-h-11"
    leadingIcon={mdiArrowLeft}
    onclick={back}
  >
    Back
  </Button>
{:else}
  <IconButton
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    class="size-11 shrink-0"
    icon={mdiArrowLeft}
    aria-label="Back"
    onclick={back}
  />
{/if}
