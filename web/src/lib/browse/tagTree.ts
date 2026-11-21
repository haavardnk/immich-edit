import type { TagSummary } from '#lib/api/tags.ts';
import { textMatches } from '#lib/utils/textMatch.ts';

export interface TagNode {
  path: string;
  name: string;
  tag: TagSummary | null;
  children: TagNode[];
}

export interface TagRow {
  node: TagNode;
  depth: number;
}

export type TagCheck = 'checked' | 'mixed' | 'unchecked';

export interface TagPath {
  parent: string;
  leaf: string;
}

export function splitTagValue(value: string): TagPath {
  const index = value.lastIndexOf('/');
  if (index < 0) return { parent: '', leaf: value };
  return { parent: value.slice(0, index), leaf: value.slice(index + 1) };
}

export interface TagChipPath extends TagPath {
  shortened: boolean;
}

export function chipTagPath(value: string): TagChipPath {
  const { parent, leaf } = splitTagValue(value);
  const direct = splitTagValue(parent).leaf;
  return { parent: direct, leaf, shortened: direct !== parent };
}

export function tagAncestors(tags: readonly TagSummary[], tag: TagSummary): TagSummary[] {
  const names = tag.value.split('/').filter((name) => name !== '');
  return names.slice(0, -1).flatMap((_, index) => {
    const value = names.slice(0, index + 1).join('/');
    const found = tags.find((candidate) => candidate.value === value);
    return found ? [found] : [];
  });
}

export function tagPathLabel(value: string): string {
  return value
    .split('/')
    .filter((name) => name !== '')
    .join(' / ');
}

export function buildTagTree(tags: readonly TagSummary[]): TagNode[] {
  const root: TagNode = { path: '', name: '', tag: null, children: [] };
  const byPath = new Map<string, TagNode>([['', root]]);
  for (const tag of tags) {
    const names = tag.value.split('/').filter((name) => name !== '');
    let parent = root;
    names.forEach((name, index) => {
      const path = names.slice(0, index + 1).join('/');
      let node = byPath.get(path);
      if (!node) {
        node = { path, name, tag: null, children: [] };
        byPath.set(path, node);
        parent.children.push(node);
      }
      if (index === names.length - 1) node.tag = tag;
      parent = node;
    });
  }
  sortNodes(root.children);
  return root.children;
}

function byName(a: TagNode, b: TagNode): number {
  return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
}

function sortNodes(nodes: TagNode[]): void {
  nodes.sort(byName);
  for (const node of nodes) sortNodes(node.children);
}

export type TagSort = 'name' | 'count';

export const TAG_SORTS: readonly { value: TagSort; label: string }[] = [
  { value: 'name', label: 'Name' },
  { value: 'count', label: 'Photo count' }
];

function photosOf(node: TagNode): number {
  return node.tag?.assetCount ?? 0;
}

const NODE_ORDER: Readonly<Record<TagSort, (a: TagNode, b: TagNode) => number>> = {
  name: byName,
  count: (a, b) => photosOf(b) - photosOf(a) || byName(a, b)
};

export function sortTagTree(nodes: readonly TagNode[], sort: TagSort): TagNode[] {
  return [...nodes]
    .sort(NODE_ORDER[sort])
    .map((node) => ({ ...node, children: sortTagTree(node.children, sort) }));
}

export function tagNodeKey(node: TagNode): string {
  return node.tag?.id ?? node.path;
}

export function branchKeys(nodes: readonly TagNode[]): string[] {
  return nodes.flatMap((node) =>
    node.children.length > 0 ? [tagNodeKey(node), ...branchKeys(node.children)] : []
  );
}

export function ancestorKeys(nodes: readonly TagNode[], tagId: string): string[] | null {
  for (const node of nodes) {
    if (node.tag?.id === tagId) return [];
    const below = ancestorKeys(node.children, tagId);
    if (below) return [tagNodeKey(node), ...below];
  }
  return null;
}

export function findTagNode(nodes: readonly TagNode[], tagId: string): TagNode | null {
  for (const node of nodes) {
    if (node.tag?.id === tagId) return node;
    const below = findTagNode(node.children, tagId);
    if (below) return below;
  }
  return null;
}

export interface TagTreeRow {
  node: TagNode;
  depth: number;
  open: boolean;
  forced: boolean;
  dimmed: boolean;
}

type IsOpen = (key: string) => boolean;

export function treeRows(nodes: readonly TagNode[], isOpen: IsOpen, depth = 0): TagTreeRow[] {
  return nodes.flatMap((node) => {
    const open = node.children.length > 0 && isOpen(tagNodeKey(node));
    const below = open ? treeRows(node.children, isOpen, depth + 1) : [];
    return [{ node, depth, open, forced: false, dimmed: false }, ...below];
  });
}

export function matchedRows(
  nodes: readonly TagNode[],
  matched: ReadonlySet<string>,
  isOpen: IsOpen,
  depth = 0
): TagTreeRow[] {
  return nodes.flatMap((node) => {
    const hit = node.tag !== null && matched.has(node.tag.id);
    const unfolded = hit && node.children.length > 0 && isOpen(tagNodeKey(node));
    const below = unfolded
      ? treeRows(node.children, isOpen, depth + 1)
      : matchedRows(node.children, matched, isOpen, depth + 1);
    if (!hit && below.length === 0) return [];
    const open = below.length > 0;
    return [{ node, depth, open, forced: open && !unfolded, dimmed: !hit }, ...below];
  });
}

export function mixedPaths(nodes: readonly TagNode[], applied: ReadonlySet<string>): Set<string> {
  const mixed = new Set<string>();
  const visit = (node: TagNode): boolean => {
    let below = false;
    for (const child of node.children) below = visit(child) || below;
    if (below) mixed.add(node.path);
    return below || (node.tag !== null && applied.has(node.tag.id));
  };
  for (const node of nodes) visit(node);
  return mixed;
}

export function tagCheck(
  node: TagNode,
  applied: ReadonlySet<string>,
  mixed: ReadonlySet<string>
): TagCheck {
  if (node.tag && applied.has(node.tag.id)) return 'checked';
  return mixed.has(node.path) ? 'mixed' : 'unchecked';
}

export function tagRows(nodes: readonly TagNode[], expanded: ReadonlySet<string>): TagRow[] {
  const rows: TagRow[] = [];
  const visit = (node: TagNode, depth: number): void => {
    rows.push({ node, depth });
    if (expanded.has(node.path)) for (const child of node.children) visit(child, depth + 1);
  };
  for (const node of nodes) visit(node, 0);
  return rows;
}

export function descendantCount(node: TagNode): number {
  return node.children.reduce((sum, child) => sum + 1 + descendantCount(child), 0);
}

export function matchTags(tags: readonly TagSummary[], query: string): TagSummary[] {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === '') return [];
  return tags
    .filter((tag) => textMatches(tag.value, query))
    .map((tag) => ({ tag, rank: matchRank(tag.value.toLocaleLowerCase(), needle) }))
    .sort((a, b) => a.rank - b.rank || a.tag.value.localeCompare(b.tag.value))
    .map(({ tag }) => tag);
}

function matchRank(value: string, needle: string): number {
  if (value === needle) return 0;
  return splitTagValue(value).leaf.startsWith(needle) ? 1 : 2;
}

export function hasExactTag(tags: readonly TagSummary[], query: string): boolean {
  const needle = query.trim().toLocaleLowerCase();
  return tags.some((tag) => tag.value.toLocaleLowerCase() === needle);
}
