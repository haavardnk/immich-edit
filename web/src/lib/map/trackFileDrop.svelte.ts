import { trackFormat } from '#lib/geotag/trackFile.ts';
import type { Attachment } from 'svelte/attachments';

function carriesFiles(event: DragEvent): boolean {
  return event.dataTransfer?.types.includes('Files') ?? false;
}

export class TrackFileDrop {
  over = $state(false);
  #onFiles: (files: File[]) => void;

  constructor(onFiles: (files: File[]) => void) {
    this.#onFiles = onFiles;
  }

  attach: Attachment<HTMLElement> = (element) => {
    const over = (event: DragEvent): void => {
      if (!carriesFiles(event)) return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
      this.over = true;
    };
    const leave = (event: DragEvent): void => {
      const box = element.getBoundingClientRect();
      const inside =
        event.clientX > box.left &&
        event.clientX < box.right &&
        event.clientY > box.top &&
        event.clientY < box.bottom;
      if (!inside) this.over = false;
    };
    const drop = (event: DragEvent): void => {
      if (!carriesFiles(event)) return;
      event.preventDefault();
      this.over = false;
      const files = [...(event.dataTransfer?.files ?? [])].filter(
        (file) => trackFormat(file.name) !== null
      );
      if (files.length > 0) this.#onFiles(files);
    };
    element.addEventListener('dragover', over);
    element.addEventListener('dragleave', leave);
    element.addEventListener('drop', drop);
    return () => {
      element.removeEventListener('dragover', over);
      element.removeEventListener('dragleave', leave);
      element.removeEventListener('drop', drop);
    };
  };
}
