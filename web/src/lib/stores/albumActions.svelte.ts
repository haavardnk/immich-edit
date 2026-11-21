import type { AlbumAction } from '#lib/browse/albumActions.ts';
import { albumDetailsPatch } from '#lib/browse/albumDetails.ts';
import type { AlbumConfirmKind } from '#lib/browse/albumDialogText.ts';
import { album } from '#lib/stores/album.svelte.ts';
import { library } from '#lib/stores/library.svelte.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { goto } from '$app/navigation';
import { page } from '$app/state';

export type AlbumDetailsField = 'name' | 'description';

export interface AlbumDetailsRequest {
  album: AlbumSummary | null;
  focus: AlbumDetailsField;
}

export interface AlbumConfirmRequest {
  kind: AlbumConfirmKind;
  album: AlbumSummary;
}

function albumPath(id: string): string {
  return `/albums/${id}`;
}

class AlbumActionsStore {
  details = $state<AlbumDetailsRequest | null>(null);
  confirming = $state<AlbumConfirmRequest | null>(null);
  saving = $state(false);

  run = async (action: AlbumAction, target: AlbumSummary): Promise<void> => {
    if (action === 'open') {
      await goto(albumPath(target.id));
      return;
    }
    if (!(await metadataConsent.gate())) return;
    if (action === 'rename') this.details = { album: target, focus: 'name' };
    else if (action === 'describe') this.details = { album: target, focus: 'description' };
    else this.confirming = { kind: action, album: target };
  };

  create = async (): Promise<void> => {
    if (!(await metadataConsent.gate())) return;
    this.details = { album: null, focus: 'name' };
  };

  closeDetails = (): void => {
    this.details = null;
  };

  save = async (name: string, description: string): Promise<void> => {
    const request = this.details;
    if (!request || this.saving) return;
    this.saving = true;
    try {
      if (request.album) await this.update(request, request.album, name, description);
      else await this.add(request, name, description);
    } finally {
      this.saving = false;
    }
  };

  confirm = async (): Promise<void> => {
    const request = this.confirming;
    if (!request) return;
    this.confirming = null;
    const target = request.album;
    const ok = request.kind === 'delete' ? await album.remove(target) : await album.leave(target);
    if (!ok) return;
    toasts.push('success', `${request.kind === 'delete' ? 'Deleted' : 'Left'} ${target.albumName}`);
    if (page.url.pathname === albumPath(target.id)) await goto('/albums');
  };

  cancel = (): void => {
    this.confirming = null;
  };

  private async update(
    request: AlbumDetailsRequest,
    target: AlbumSummary,
    name: string,
    description: string
  ): Promise<void> {
    const patch = albumDetailsPatch(target, name, description);
    const saved = patch ? await album.update(target.id, patch) : target;
    if (saved && this.details === request) this.details = null;
  }

  private async add(
    request: AlbumDetailsRequest,
    name: string,
    description: string
  ): Promise<void> {
    const created = await library.createAlbum(name.trim(), description.trim());
    if (!created || this.details !== request) return;
    this.details = null;
    await goto(albumPath(created.id));
  }
}

export const albumActions = new AlbumActionsStore();
