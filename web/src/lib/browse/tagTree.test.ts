import type { TagSummary } from '#lib/api/tags.ts';
import {
  ancestorKeys,
  branchKeys,
  buildTagTree,
  chipTagPath,
  descendantCount,
  findTagNode,
  hasExactTag,
  matchedRows,
  matchTags,
  mixedPaths,
  sortTagTree,
  splitTagValue,
  tagAncestors,
  tagCheck,
  tagPathLabel,
  tagRows,
  treeRows,
  type TagNode,
  type TagTreeRow
} from '#lib/browse/tagTree.ts';
import { describe, expect, it } from 'vitest';

function tag(id: string, value: string): TagSummary {
  return { id, name: value.split('/').pop() ?? value, value, createdAt: null };
}

const TAGS = [
  tag('lofoten', 'Places/Norway/Lofoten'),
  tag('places', 'Places'),
  tag('senja', 'Places/Norway/Senja'),
  tag('norway', 'Places/Norway'),
  tag('iceland', 'Places/Iceland'),
  tag('fjord', 'Subjects/Fjord'),
  tag('subjects', 'Subjects'),
  tag('aurora', 'aurora')
];

function find(nodes: readonly TagNode[], path: string): TagNode {
  for (const node of nodes) {
    if (node.path === path) return node;
    if (path.startsWith(`${node.path}/`)) return find(node.children, path);
  }
  throw new Error(`no ${path}`);
}

describe('splitTagValue', () => {
  it('splits the parent path from the leaf', () => {
    expect(splitTagValue('Places/Norway/Lofoten')).toEqual({
      parent: 'Places/Norway',
      leaf: 'Lofoten'
    });
    expect(splitTagValue('aurora')).toEqual({ parent: '', leaf: 'aurora' });
  });
});

describe('chipTagPath', () => {
  it('keeps only the direct parent and marks deeper paths as shortened', () => {
    expect(chipTagPath('Places/Norway/Lofoten')).toEqual({
      parent: 'Norway',
      leaf: 'Lofoten',
      shortened: true
    });
    expect(chipTagPath('Subjects/Fjord')).toEqual({
      parent: 'Subjects',
      leaf: 'Fjord',
      shortened: false
    });
    expect(chipTagPath('aurora')).toEqual({ parent: '', leaf: 'aurora', shortened: false });
  });
});

describe('tagPathLabel', () => {
  it('joins every level with one spaced separator', () => {
    expect(tagPathLabel('Places/Norway/Lofoten')).toBe('Places / Norway / Lofoten');
    expect(tagPathLabel('aurora')).toBe('aurora');
  });
});

describe('buildTagTree', () => {
  it('nests tags by their path and sorts every level by name', () => {
    const tree = buildTagTree(TAGS);
    expect(tree.map((node) => node.name)).toEqual(['aurora', 'Places', 'Subjects']);
    expect(find(tree, 'Places').children.map((node) => node.name)).toEqual(['Iceland', 'Norway']);
    expect(find(tree, 'Places/Norway/Lofoten').tag?.id).toBe('lofoten');
  });

  it('keeps a parent the library does not list as a node without a tag', () => {
    const tree = buildTagTree([tag('red', 'Colours/Red')]);
    expect(tree[0]?.tag).toBeNull();
    expect(tree[0]?.children[0]?.tag?.id).toBe('red');
  });
});

describe('tagCheck', () => {
  const tree = buildTagTree(TAGS);
  const applied = new Set(['lofoten', 'fjord']);
  const mixed = mixedPaths(tree, applied);

  it('checks applied tags and marks their ancestors as mixed', () => {
    expect(tagCheck(find(tree, 'Places/Norway/Lofoten'), applied, mixed)).toBe('checked');
    expect(tagCheck(find(tree, 'Places/Norway'), applied, mixed)).toBe('mixed');
    expect(tagCheck(find(tree, 'Places'), applied, mixed)).toBe('mixed');
    expect(tagCheck(find(tree, 'Places/Iceland'), applied, mixed)).toBe('unchecked');
  });

  it('checks an applied parent even when a child is applied too', () => {
    const both = new Set(['norway', 'lofoten']);
    expect(tagCheck(find(tree, 'Places/Norway'), both, mixedPaths(tree, both))).toBe('checked');
  });
});

describe('tagRows', () => {
  it('lists only the children of expanded nodes, with their depth', () => {
    const tree = buildTagTree(TAGS);
    const rows = tagRows(tree, new Set(['Places', 'Places/Norway']));
    expect(rows.map((row) => `${row.depth}:${row.node.name}`)).toEqual([
      '0:aurora',
      '0:Places',
      '1:Iceland',
      '1:Norway',
      '2:Lofoten',
      '2:Senja',
      '0:Subjects'
    ]);
  });
});

describe('descendantCount', () => {
  it('counts every node below', () => {
    expect(descendantCount(find(buildTagTree(TAGS), 'Places'))).toBe(4);
  });
});

describe('matchTags', () => {
  it('matches anywhere in the path, ignoring case and accents', () => {
    const tags = [...TAGS, tag('lan', 'Lån')];
    expect(matchTags(tags, 'lan').map((t) => t.id)).toEqual(['lan', 'iceland']);
    expect(matchTags(tags, 'NOR').map((t) => t.id)).toEqual(['norway', 'lofoten', 'senja']);
  });

  it('ranks an exact value, then a leaf prefix, before other matches', () => {
    expect(matchTags(TAGS, 'places').map((t) => t.id)).toEqual([
      'places',
      'iceland',
      'norway',
      'lofoten',
      'senja'
    ]);
    expect(matchTags(TAGS, 'fj').map((t) => t.id)).toEqual(['fjord']);
  });

  it('matches nothing for a blank query', () => {
    expect(matchTags(TAGS, '  ')).toEqual([]);
  });
});

describe('hasExactTag', () => {
  it('compares the whole value without case', () => {
    expect(hasExactTag(TAGS, ' subjects/FJORD ')).toBe(true);
    expect(hasExactTag(TAGS, 'Fjord')).toBe(false);
  });
});

function counted(id: string, value: string, assetCount: number): TagSummary {
  return { ...tag(id, value), assetCount };
}

function names(rows: readonly TagTreeRow[]): string[] {
  return rows.map(({ node, depth, open, forced, dimmed }) =>
    [' '.repeat(depth * 2), node.name, open ? (forced ? ' =' : ' v') : '', dimmed ? ' ~' : ''].join(
      ''
    )
  );
}

describe('tagAncestors', () => {
  it('lists the tags above a tag from the top down and skips missing levels', () => {
    const senja = TAGS.find((t) => t.id === 'senja');
    if (!senja) throw new Error('senja missing');
    expect(tagAncestors(TAGS, senja).map((t) => t.id)).toEqual(['places', 'norway']);
    const orphan = tag('orphan', 'Gone/Orphan');
    expect(tagAncestors(TAGS, orphan)).toEqual([]);
  });
});

describe('library tree', () => {
  const tree = buildTagTree(TAGS);
  const openOnly =
    (...keys: string[]) =>
    (key: string): boolean =>
      keys.includes(key);

  it('sorts siblings by photo count within their parent, names breaking ties', () => {
    const sorted = sortTagTree(
      buildTagTree([
        counted('b', 'B', 5),
        counted('a', 'A', 5),
        counted('c', 'C', 9),
        counted('c1', 'C/One', 1),
        counted('c2', 'C/Two', 4)
      ]),
      'count'
    );
    expect(sorted.map((node) => node.name)).toEqual(['C', 'A', 'B']);
    expect(sorted[0]?.children.map((node) => node.name)).toEqual(['Two', 'One']);
  });

  it('lists rows under open branches only', () => {
    expect(names(treeRows(tree, openOnly('places')))).toEqual([
      'aurora',
      'Places v',
      '  Iceland',
      '  Norway',
      'Subjects'
    ]);
  });

  it('keys branches by tag id and finds the ancestors of a tag', () => {
    expect(branchKeys(tree)).toEqual(['places', 'norway', 'subjects']);
    expect(ancestorKeys(tree, 'senja')).toEqual(['places', 'norway']);
    expect(ancestorKeys(tree, 'aurora')).toEqual([]);
    expect(ancestorKeys(tree, 'missing')).toBeNull();
    expect(findTagNode(tree, 'norway')?.children.length).toBe(2);
  });

  it('shows matches under dimmed open ancestors and keeps a matched parent folded', () => {
    const rows = matchedRows(tree, new Set(['senja', 'norway', 'subjects']), openOnly());
    expect(names(rows)).toEqual(['Places = ~', '  Norway =', '    Senja', 'Subjects']);
  });

  it('unfolds a matched parent the user opened while filtering', () => {
    const rows = matchedRows(tree, new Set(['subjects']), openOnly('subjects'));
    expect(names(rows)).toEqual(['Subjects v', '  Fjord']);
  });
});
