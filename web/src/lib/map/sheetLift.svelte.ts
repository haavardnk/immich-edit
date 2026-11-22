import type { Attachment } from 'svelte/attachments';

export class SheetLift {
  value = $state(0);
  area: HTMLElement | null = null;

  attach: Attachment<HTMLElement> = (area) => {
    this.area = area;
    const update = (): void => {
      this.value = Math.max(
        0,
        Math.round(window.innerHeight - area.getBoundingClientRect().bottom)
      );
    };
    const observer = new ResizeObserver(update);
    observer.observe(area);
    observer.observe(document.body);
    window.addEventListener('resize', update);
    update();
    return () => {
      this.area = null;
      observer.disconnect();
      window.removeEventListener('resize', update);
    };
  };
}
