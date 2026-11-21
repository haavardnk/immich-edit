import type { MockTag } from './helpers';

function tag(id: string, value: string, assetCount: number, extra: Partial<MockTag> = {}): MockTag {
  return { id, name: value.split('/').at(-1) ?? value, value, assetCount, ...extra };
}

export const TAG_TREE: readonly MockTag[] = [
  tag('travel', 'Travel', 1290, { color: '#3e8ed0' }),
  tag('germany', 'Travel/Germany', 92, { parentId: 'travel' }),
  tag('berlin', 'Travel/Germany/Berlin', 48, { parentId: 'germany' }),
  tag('norway', 'Travel/Norway', 640, { parentId: 'travel' }),
  tag('bergen', 'Travel/Norway/Bergen', 54, { parentId: 'norway' }),
  tag('lofoten', 'Travel/Norway/Lofoten', 186, { parentId: 'norway' }),
  tag('family', 'Family', 340, { color: '#f5c542' }),
  tag('film', 'Film', 220),
  tag('berries', 'Berries', 18),
  tag('reject', 'immich-edit/reject', 7)
];

export function tagTree(): MockTag[] {
  return TAG_TREE.map((entry) => ({ ...entry }));
}
