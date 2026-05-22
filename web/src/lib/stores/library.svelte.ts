import { listAlbums } from '$lib/api/albums';
import { listPeople, type PersonSummary } from '$lib/api/people';
import { listTags, type TagSummary } from '$lib/api/tags';
import { folderPaths } from '$lib/api/folders';
import { listEditedAssetIds } from '$lib/api/edits';
import { assetStatistics } from '$lib/api/search';
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

  photosCount = $state<number | null>(null);
  favoritesCount = $state<number | null>(null);
  editedCount = $state<number | null>(null);
  private folderPathCount = $state<number | null>(null);

  get foldersCount(): number | null {
    return this.folderPathCount;
  }

  private loaded = new Set<LibraryView>();
  private countsLoaded = false;

  async loadView(v: LibraryView): Promise<void> {
    this.view = v;
    if (this.loaded.has(v) || this.loading) return;
    this.loading = true;
    this.error = null;
    try {
      switch (v) {
        case 'albums': {
          const a = await listAlbums();
          this.albums = a.sort((x, y) => x.albumName.localeCompare(y.albumName));
          break;
        }
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

  async loadCounts(): Promise<void> {
    if (this.countsLoaded) return;
    this.countsLoaded = true;
    const [stats, favStats, albums, people, tags, paths, editedIds] = await Promise.all([
      assetStatistics().catch(() => null),
      assetStatistics({ isFavorite: 'true' }).catch(() => null),
      listAlbums().catch(() => [] as AlbumSummary[]),
      listPeople().catch(() => [] as PersonSummary[]),
      listTags().catch(() => [] as TagSummary[]),
      folderPaths().catch(() => [] as string[]),
      listEditedAssetIds().catch(() => [] as string[]),
    ]);
    if (stats) this.photosCount = stats.total;
    if (favStats) this.favoritesCount = favStats.total;
    if (!this.loaded.has('albums')) {
      this.albums = albums.sort((x, y) => x.albumName.localeCompare(y.albumName));
      this.loaded.add('albums');
    }
    if (!this.loaded.has('people')) {
      this.people = people;
      this.loaded.add('people');
    }
    if (!this.loaded.has('tags')) {
      this.tags = tags;
      this.loaded.add('tags');
    }
    if (!this.loaded.has('folders')) {
      this.folderTree = buildTree(paths);
      this.loaded.add('folders');
    }
    this.folderPathCount = paths.length;
    this.editedCount = editedIds.length;
  }
}

export const library = new LibraryStore();
