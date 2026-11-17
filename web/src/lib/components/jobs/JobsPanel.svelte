<script lang="ts">
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import JobList from '#lib/components/jobs/JobList.svelte';
  import JobListActions from '#lib/components/jobs/JobListActions.svelte';
  import JobNotifyToggle from '#lib/components/jobs/JobNotifyToggle.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { onMount } from 'svelte';
  import { on } from 'svelte/events';

  let { class: className = '' }: { class?: string } = $props();

  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;

  let panel = $state<HTMLElement | null>(null);

  onMount(() => panel?.focus());

  function close(): void {
    const inside = panel?.contains(document.activeElement) ?? false;
    jobs.close();
    if (inside && opener?.isConnected) opener.focus();
  }

  function closeOnEscape(node: HTMLElement): () => void {
    return on(node, 'keydown', (event) => {
      if (event.key !== 'Escape' || event.defaultPrevented) return;
      event.preventDefault();
      event.stopPropagation();
      close();
    });
  }
</script>

<aside
  bind:this={panel}
  id="jobs-panel"
  aria-labelledby="jobs-panel-title"
  tabindex="-1"
  class="flex h-full w-100 max-w-[92vw] flex-col border-s border-hairline bg-light-50 outline-none {className}"
  {@attach closeOnEscape}
>
  <header class="flex h-14 flex-none items-center gap-2 border-b border-hairline ps-4 pe-2">
    <h2 id="jobs-panel-title" class="text-[15px] font-semibold text-light-950">Jobs</h2>
    <div class="ms-auto flex items-center gap-1">
      <JobNotifyToggle />
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="touch:size-11"
        icon={mdiClose}
        aria-label="Close jobs"
        onclick={close}
      />
    </div>
  </header>
  <div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
    <JobList grouped={layout.mode === 'touch'} />
  </div>
  {#if jobs.clearableCount > 0}
    <footer class="flex-none border-t border-hairline p-3">
      <JobListActions />
    </footer>
  {/if}
</aside>
