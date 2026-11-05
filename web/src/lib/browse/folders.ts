import type { FolderNode } from '#lib/stores/library.svelte.ts';

type FolderCrumb = { name: string; path: string };

export function folderHref(path: string): string {
  return path ? `/folders?path=${encodeURIComponent(path)}` : '/folders';
}

export function folderCrumbs(path: string): FolderCrumb[] {
  const parts = path.split('/').filter(Boolean);
  return parts.map((name, index) => ({ name, path: `/${parts.slice(0, index + 1).join('/')}` }));
}

export function parentFolder(path: string): string {
  return folderCrumbs(path).at(-2)?.path ?? '';
}

export function folderName(path: string): string {
  return folderCrumbs(path).at(-1)?.name ?? '';
}

export function findFolder(nodes: readonly FolderNode[], path: string): FolderNode | null {
  for (const node of nodes) {
    if (node.path === path) return node;
    if (path.startsWith(`${node.path}/`)) return findFolder(node.children, path);
  }
  return null;
}

export function subfolders(nodes: readonly FolderNode[], path: string): readonly FolderNode[] {
  if (!path) return nodes;
  return findFolder(nodes, path)?.children ?? [];
}
