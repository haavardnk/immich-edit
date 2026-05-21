import { listAlbums } from '$lib/api/albums';
import { listPeople, type PersonSummary } from '$lib/api/people';
import { listTags, type TagSummary } from '$lib/api/tags';
import { folderPaths } from '$lib/api/folders';
import type { AlbumSummary } from '$lib/types/album';

export type LibraryView = 'albums' | 'folders' | 'people' | 'favorites' | 'tags';

export interface FolderNode {
  name: string;
  path: string;
  children: FolderNode[];
}

function buildTree(paths: string[]): FolderNode[] {
  const root: FolderNode = { name: '', path: '', children: [] };
  for (const p of paths) {
    const parts = p.replace(/^\//, '').split('/');
    let node = root;
    let acc = '';
    for (const part of parts) {
      acc = acc ? `${acc}/${part}` : `/${part}`;
      let child = node.children.find((c) => c.name === part);
      if (!child) {
        child = { name: part, path: acc, children: [] };
        node.children.push(child);
      }
      node = child;
    }
  }
  return root.children;
}

class LibraryStore {
  view = $state<LibraryView>('albums');
  albums = $state<AlbumSummary[]>([]);
  people = $state<PersonSummary[]>([]);
  tags = $state<TagSummary[]>([]);
  folderTree = $state<FolderNode[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);

  private loaded = new Set<LibraryView>();

  async loadView(v: LibraryView): Promise<void> {
    this.view = v;
    if (this.loaded.has(v) || this.loading) return;
    this.loading = true;
    this.error = null;
    try {
      switch (v) {
        case 'albums':
          this.albums = await listAlbums();
          break;
        case 'people':
          this.people = await listPeople();
          break;
        case 'tags':
          this.tags = await listTags();
          break;
        case 'folders':
          this.folderTree = buildTree(await folderPaths());
          break;
        case 'favorites':
          break;
      }
      this.loaded.add(v);
    } catch (e) {
      this.error = (e as Error).message;
    } finally {
      this.loading = false;
    }
  }

  async load(force = false): Promise<void> {
    if (force) this.loaded.delete(this.view);
    await this.loadView(this.view);
  }
}

export const library = new LibraryStore();
