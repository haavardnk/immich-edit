import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import { createAlbum, listAlbums } from '#lib/api/albums.ts';
import { listPeople, type PersonSummary } from '#lib/api/people.ts';
import { listTags, type TagSummary } from '#lib/api/tags.ts';
import { folderPaths } from '#lib/api/folders.ts';
import { searchStatistics } from '#lib/api/search.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { recentTags } from '#lib/stores/recentTags.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { errorMessage } from '#lib/utils/errors.ts';

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

function byName(albums: AlbumSummary[]): AlbumSummary[] {
  return albums.sort((x, y) => x.albumName.localeCompare(y.albumName));
}

class LibraryStore {
  view = $state<LibraryView>('albums');
  albums = $state<AlbumSummary[]>([]);
  people = $state<PersonSummary[]>([]);
  tags = $state<TagSummary[]>([]);
  folderTree = $state<FolderNode[]>([]);

  photosCount = $state<number | null>(null);
  favoritesCount = $state<number | null>(null);
  private folderPathCount = $state<number | null>(null);
  private editedCountLoaded = $state(false);

  private loaded = new SvelteSet<LibraryView>();
  private errors = new SvelteMap<LibraryView, string>();
  private pending = new Map<LibraryView, Promise<void>>();
  private countsLoading: Promise<void> | null = null;

  get foldersCount(): number | null {
    return this.loaded.has('folders') ? this.folderPathCount : null;
  }

  get editedCount(): number | null {
    return this.editedCountLoaded ? editedThumbs.count : null;
  }

  isLoaded(view: LibraryView): boolean {
    return this.loaded.has(view);
  }

  errorFor(view: LibraryView): string | null {
    return this.errors.get(view) ?? null;
  }

  ensure(view: LibraryView): Promise<void> {
    if (this.loaded.has(view)) return Promise.resolve();
    const pending = this.pending.get(view);
    if (pending) return pending;
    this.errors.delete(view);
    const loading = this.fetch(view)
      .then(() => {
        this.loaded.add(view);
      })
      .catch((e: unknown) => {
        this.errors.set(view, errorMessage(e));
        throw e;
      })
      .finally(() => {
        this.pending.delete(view);
      });
    this.pending.set(view, loading);
    return loading;
  }

  request = (view: LibraryView): void => {
    this.ensure(view).catch(() => undefined);
  };

  createAlbum = async (name: string): Promise<AlbumSummary | null> => {
    try {
      const album = await createAlbum(name);
      this.albums = byName([...this.albums, album]);
      return album;
    } catch (e: unknown) {
      toasts.fail('Failed to create album', e, 8000);
      return null;
    }
  };

  ensureTags(): Promise<void> {
    return this.ensure('tags');
  }

  async loadView(v: LibraryView): Promise<void> {
    this.view = v;
    await this.ensure(v).catch(() => undefined);
  }

  async load(force = false): Promise<void> {
    if (force) this.loaded.delete(this.view);
    await this.loadView(this.view);
  }

  loadCounts(): Promise<void> {
    this.countsLoading ??= this.fetchCounts().catch(() => {
      this.countsLoading = null;
    });
    return this.countsLoading;
  }

  private async fetch(view: LibraryView): Promise<void> {
    switch (view) {
      case 'albums':
        this.albums = byName(await listAlbums());
        break;
      case 'people':
        this.people = await listPeople();
        break;
      case 'tags':
        this.tags = await listTags();
        recentTags.keep(this.tags);
        break;
      case 'folders': {
        const paths = await folderPaths();
        this.folderTree = buildTree(paths);
        this.folderPathCount = paths.length;
        break;
      }
      case 'favorites':
        break;
    }
  }

  private async fetchCounts(): Promise<void> {
    const views = (['albums', 'people', 'tags', 'folders'] as const).map((view) =>
      this.ensure(view).catch(() => undefined)
    );
    const [stats, favStats] = await Promise.all([
      searchStatistics({}),
      searchStatistics({ favorite: true }),
      editedThumbs.loadOnce().then(() => {
        this.editedCountLoaded = true;
      }),
      ...views
    ]);
    this.photosCount = stats.total;
    this.favoritesCount = favStats.total;
  }
}

export const library = new LibraryStore();
