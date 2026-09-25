import { readStored, writeStored } from '$lib/utils/storage';

export type SidebarSectionId = 'people' | 'albums' | 'tags' | 'folders';

const SECTIONS: readonly SidebarSectionId[] = ['people', 'albums', 'tags', 'folders'];
const STORAGE_KEY = 'immich-edit:sidebar';

type Persisted = { expanded: SidebarSectionId[] };

export function sectionForPath(pathname: string): SidebarSectionId | null {
  if (pathname.startsWith('/people/')) return 'people';
  if (pathname.startsWith('/albums/')) return 'albums';
  if (pathname.startsWith('/tags/')) return 'tags';
  if (pathname === '/folders') return 'folders';
  return null;
}

class SidebarStore {
  expanded = $state<SidebarSectionId[]>([]);

  constructor() {
    const stored = readStored<Persisted>(STORAGE_KEY)?.expanded;
    if (Array.isArray(stored)) this.expanded = SECTIONS.filter((id) => stored.includes(id));
  }

  isOpen(id: SidebarSectionId): boolean {
    return this.expanded.includes(id);
  }

  toggle(id: SidebarSectionId): boolean {
    const open = !this.isOpen(id);
    this.write(open ? [...this.expanded, id] : this.expanded.filter((s) => s !== id));
    return open;
  }

  reveal(pathname: string): void {
    const id = sectionForPath(pathname);
    if (id && !this.isOpen(id)) this.write([...this.expanded, id]);
  }

  private write(expanded: SidebarSectionId[]): void {
    this.expanded = expanded;
    writeStored(STORAGE_KEY, { expanded } satisfies Persisted);
  }
}

export const sidebar = new SidebarStore();
