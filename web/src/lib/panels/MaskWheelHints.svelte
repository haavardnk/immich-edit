<script lang="ts">
  import { keysFor } from '#lib/shortcuts/labels.ts';

  const ROWS = [
    { keys: 'maskShapeSizeWheel', label: 'Size' },
    { keys: 'maskShapeFeatherWheel', label: 'Feather' }
  ] as const;
</script>

<div class="mt-1 flex flex-col gap-1 touch:hidden" data-testid="mask-wheel-hints">
  <div class="text-[10px] font-medium tracking-wider text-dark/65 uppercase">Over the photo</div>
  <dl class="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-x-2 gap-y-1 text-[11px]">
    {#each ROWS as row (row.keys)}
      <dt>
        <kbd
          class="rounded border border-hairline px-1.5 py-0.5 font-mono text-[10px] text-dark/65"
        >
          {keysFor(row.keys)}
        </kbd>
      </dt>
      <dd class="text-dark/65">{row.label}</dd>
    {/each}
  </dl>
</div>
