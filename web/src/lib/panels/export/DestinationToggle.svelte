<script lang="ts">
  import type { Destination } from './settings';

  let {
    value = $bindable<Destination>(),
    downloadLabel = 'Download',
    immichLabel = 'To Immich'
  }: {
    value: Destination;
    downloadLabel?: string;
    immichLabel?: string;
  } = $props();
</script>

<div class="flex gap-1 p-0.5 rounded-lg bg-immich-dark-bg/40">
  <button
    class="flex-1 py-1.5 text-xs rounded transition-colors"
    class:bg-immich-dark-primary={value === 'download'}
    class:text-immich-dark-bg={value === 'download'}
    class:text-immich-dark-fg={value !== 'download'}
    onclick={() => (value = 'download')}
  >
    {downloadLabel}
  </button>
  <button
    class="flex-1 py-1.5 text-xs rounded transition-colors"
    class:bg-immich-dark-primary={value === 'immich'}
    class:text-immich-dark-bg={value === 'immich'}
    class:text-immich-dark-fg={value !== 'immich'}
    onclick={() => (value = 'immich')}
  >
    {immichLabel}
  </button>
</div>
