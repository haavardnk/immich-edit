import { otherMembers } from '#lib/browse/albumRole.ts';
import { albumPhotos, ownerName } from '#lib/browse/albumSharing.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

export type AlbumTextPart = string | { strong: string };

export interface AlbumDialogText {
  title: string;
  confirm: string;
  lines: AlbumTextPart[][];
}

export type AlbumConfirmKind = 'delete' | 'leave';

function keptPhotos(count: number): string {
  if (count === 0) return '. It has no photos.';
  const verb = count === 1 ? 'stays' : 'stay';
  return `. Its ${albumPhotos(count)} ${verb} in your library and in any other albums.`;
}

function firstName(name: string): string {
  return name.split(/\s+/)[0] ?? name;
}

export function deleteAlbumText(album: AlbumSummary, userId: string | null): AlbumDialogText {
  const others = otherMembers(album, userId);
  const lines: AlbumTextPart[][] = [
    ['Immich deletes the album ', { strong: album.albumName }, keptPhotos(album.assetCount)]
  ];
  if (others > 0) {
    lines.push([
      `The album is shared with ${countOf(others, 'person', 'people')}. It disappears for them too, and its shared links stop working.`
    ]);
  }
  lines.push(['This cannot be undone.']);
  return { title: 'Delete album', confirm: 'Delete album', lines };
}

export function leaveAlbumText(album: AlbumSummary): AlbumDialogText {
  const owner = ownerName(album);
  const by = owner ? `, shared by ${owner}` : '';
  const short = owner ? firstName(owner) : 'the owner';
  return {
    title: 'Leave album',
    confirm: 'Leave album',
    lines: [
      [
        'You leave ',
        { strong: album.albumName },
        `${by}. It disappears from your albums; ${short} and the other members keep it.`
      ],
      [`Only ${short} can invite you back.`]
    ]
  };
}
