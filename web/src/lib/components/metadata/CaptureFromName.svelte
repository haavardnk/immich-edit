<script lang="ts">
  import { NO_DATE, captureEditor } from '#lib/stores/captureEditor.svelte.ts';

  const editor = captureEditor;
</script>

<ul
  class="flex flex-col overflow-hidden rounded-lg bg-light-100 ring-1 ring-hairline"
  aria-label="Dates found in file names"
>
  {#each editor.nameSources as { source, count } (source)}
    <li
      class="flex items-center gap-3 border-t border-hairline px-3 py-2 text-xs first:border-t-0 touch:py-3 touch:text-sm {source ===
      NO_DATE
        ? 'text-dark/65'
        : 'text-light-950'}"
    >
      <span class="min-w-0 flex-1 truncate">{source}</span>
      <span class="shrink-0 text-dark/65 tabular-nums">{count}</span>
    </li>
  {/each}
</ul>
