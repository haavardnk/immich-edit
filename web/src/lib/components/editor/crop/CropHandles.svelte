<script lang="ts">
  import { CROP_HANDLES } from '#lib/editor/handles.ts';
  import type { CropHandle } from '#lib/utils/geom.ts';

  let {
    hit,
    touch,
    active,
    onbegin
  }: {
    hit: number;
    touch: boolean;
    active: CropHandle | null;
    onbegin: (e: PointerEvent, handle: CropHandle) => void;
  } = $props();

  const BAR = 3;
  const length = $derived(touch ? 22 : 16);

  const CURSORS: Record<CropHandle, string> = {
    n: 'ns-resize',
    s: 'ns-resize',
    e: 'ew-resize',
    w: 'ew-resize',
    nw: 'nwse-resize',
    se: 'nwse-resize',
    ne: 'nesw-resize',
    sw: 'nesw-resize'
  };

  function place(h: CropHandle): string {
    return [
      h.includes('n') ? `top: -${hit}px;` : '',
      h.includes('s') ? `bottom: -${hit}px;` : '',
      h.includes('w') ? `left: -${hit}px;` : '',
      h.includes('e') ? `right: -${hit}px;` : '',
      h === 'n' || h === 's' ? `left: calc(50% - ${hit}px);` : '',
      h === 'w' || h === 'e' ? `top: calc(50% - ${hit}px);` : ''
    ].join(' ');
  }

  function mark(h: CropHandle): string {
    const edge = hit - BAR;
    const along = `calc(50% - ${length / 2}px)`;
    if (h === 'n' || h === 's') {
      return `${h === 'n' ? 'top' : 'bottom'}: ${edge}px; left: ${along}; width: ${length}px; height: ${BAR}px;`;
    }
    if (h === 'w' || h === 'e') {
      return `${h === 'w' ? 'left' : 'right'}: ${edge}px; top: ${along}; width: ${BAR}px; height: ${length}px;`;
    }
    const vertical = h.includes('n') ? 'top' : 'bottom';
    const horizontal = h.includes('w') ? 'left' : 'right';
    return `${vertical}: ${edge}px; ${horizontal}: ${edge}px; width: ${length}px; height: ${length}px; border-${vertical}-width: ${BAR}px; border-${horizontal}-width: ${BAR}px; transform-origin: ${vertical} ${horizontal};`;
  }
</script>

{#each CROP_HANDLES as h (h)}
  {@const corner = h.length === 2}
  <button
    class="group absolute touch-none"
    style="width: {hit * 2}px; height: {hit * 2}px; cursor: {CURSORS[h]}; {place(h)}"
    data-active={active === h ? '' : undefined}
    onpointerdown={(e) => onbegin(e, h)}
    aria-label="resize {h}"
  >
    <span
      class="pointer-events-none absolute transition-[scale,opacity,background-color,border-color] duration-150 group-hover:scale-125 group-data-active:scale-125 {corner
        ? 'border-0 border-white group-hover:border-image-accent group-data-active:border-image-accent'
        : 'bg-white group-hover:bg-image-accent group-data-active:bg-image-accent'} {corner || touch
        ? ''
        : 'opacity-0 group-hover:opacity-100 group-data-active:opacity-100'}"
      style={mark(h)}
    ></span>
  </button>
{/each}
