<script lang="ts">
  import { Icon } from '@immich/ui';
  import { mdiChevronUp } from '@mdi/js';

  let { label, onOpen }: { label: string; onOpen: () => void } = $props();
</script>

<button
  type="button"
  class="absolute bottom-3 left-1/2 z-10 flex h-11 max-w-[calc(100%-2rem)] -translate-x-1/2 items-center gap-2 rounded-full bg-light-100 ps-4 pe-3 text-sm font-medium text-light-950 shadow-popover ring-1 ring-hairline"
  aria-label="Show {label}"
  data-map-pill
  onclick={onOpen}
>
  <span class="truncate">{label}</span>
  <Icon icon={mdiChevronUp} size="16px" class="shrink-0 text-dark/65" aria-hidden="true" />
</button>
