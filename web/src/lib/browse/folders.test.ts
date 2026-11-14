import { describe, expect, it } from 'vitest';
import type { FolderNode } from '#lib/stores/library.svelte.ts';
import {
  activeFolderPath,
  allFolders,
  findFolder,
  folderCrumbs,
  folderHref,
  folderName,
  parentFolder,
  subfolders
} from './folders';

const TREE: FolderNode[] = [
  {
    name: 'photos',
    path: '/photos',
    children: [
      {
        name: '2026',
        path: '/photos/2026',
        children: [{ name: 'Norway', path: '/photos/2026/Norway', children: [] }]
      }
    ]
  },
  { name: 'phone', path: '/phone', children: [] }
];

describe('folder paths', () => {
  it('splits a path into crumbs that link to each level', () => {
    expect(folderCrumbs('/photos/2026/Norway')).toEqual([
      { name: 'photos', path: '/photos' },
      { name: '2026', path: '/photos/2026' },
      { name: 'Norway', path: '/photos/2026/Norway' }
    ]);
    expect(folderCrumbs('')).toEqual([]);
  });

  it('names the current folder', () => {
    expect(folderName('/photos/2026/Norway')).toBe('Norway');
    expect(folderName('')).toBe('');
  });

  it('finds the folder one level up', () => {
    expect(parentFolder('/photos/2026/Norway')).toBe('/photos/2026');
    expect(parentFolder('/photos')).toBe('');
    expect(parentFolder('')).toBe('');
  });

  it('builds folder links', () => {
    expect(folderHref('/photos/a b')).toBe('/folders?path=%2Fphotos%2Fa%20b');
    expect(folderHref('')).toBe('/folders');
  });
});

describe('folder tree lookup', () => {
  it('finds a nested folder without matching a sibling prefix', () => {
    expect(findFolder(TREE, '/photos/2026')?.name).toBe('2026');
    expect(findFolder(TREE, '/pho')).toBeNull();
    expect(findFolder(TREE, '/phone')?.name).toBe('phone');
  });

  it('lists the subfolders of a path, or the roots', () => {
    expect(subfolders(TREE, '').map((node) => node.name)).toEqual(['photos', 'phone']);
    expect(subfolders(TREE, '/photos/2026').map((node) => node.name)).toEqual(['Norway']);
    expect(subfolders(TREE, '/missing')).toEqual([]);
  });

  it('lists every folder of the tree, parents before their children', () => {
    expect(allFolders(TREE).map((node) => node.path)).toEqual([
      '/photos',
      '/photos/2026',
      '/photos/2026/Norway',
      '/phone'
    ]);
  });
});

describe('activeFolderPath', () => {
  it('reads the folder only on the folders page', () => {
    expect(activeFolderPath(new URL('http://x/folders?path=%2Fphotos%2F2026'))).toBe(
      '/photos/2026'
    );
    expect(activeFolderPath(new URL('http://x/folders'))).toBeNull();
    expect(activeFolderPath(new URL('http://x/photos?path=%2Fphotos'))).toBeNull();
  });
});
