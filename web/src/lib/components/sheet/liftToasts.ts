import type { Attachment } from 'svelte/attachments';

const TOAST_OFFSET = '--chrome-bottom';

export const liftToasts: Attachment<HTMLElement> = (element) => {
  const root = document.documentElement.style;
  const previous = root.getPropertyValue(TOAST_OFFSET);
  const update = (): void => root.setProperty(TOAST_OFFSET, `${element.offsetHeight}px`);
  const observer = new ResizeObserver(update);
  observer.observe(element);
  update();
  return () => {
    observer.disconnect();
    if (previous) root.setProperty(TOAST_OFFSET, previous);
    else root.removeProperty(TOAST_OFFSET);
  };
};
