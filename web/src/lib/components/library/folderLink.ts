export function folderLinkClass(active: boolean): string {
  return `flex items-center rounded-e-full py-1 pe-4 outline-none transition-colors focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary ${
    active ? 'bg-primary/15 text-primary' : 'hover:bg-subtle hover:text-primary'
  }`;
}
