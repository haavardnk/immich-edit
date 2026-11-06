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

function sortNodes(nodes: TagNode[]): void {
  nodes.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
  for (const node of nodes) sortNodes(node.children);
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
