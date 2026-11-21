import { canSetViewedAlbumCover, setViewedAlbumCover } from '#lib/browse/albumCover.ts';
import { loupeFocusedId } from '#lib/browse/loupeActions.ts';
import { editableViewedAlbum } from '#lib/browse/viewedAlbum.ts';
import { loupeCommand } from '#lib/commands/loupe/context.ts';
import type { Command } from '#lib/commands/types.ts';
import { mdiImageOutline } from '@mdi/js';

export const loupeSetAlbumCover = loupeCommand({
  id: 'loupe.setAlbumCover',
  contexts: ['loupe'],
  label: 'Set as album cover',
  icon: mdiImageOutline,
  shown: () => editableViewedAlbum() !== null,
  enabled: () => canSetViewedAlbumCover(loupeFocusedId()),
  run: () => {
    const id = loupeFocusedId();
    if (id) void setViewedAlbumCover(id);
  }
});

export const LOUPE_ALBUM_COMMANDS: readonly Command[] = [loupeSetAlbumCover];
