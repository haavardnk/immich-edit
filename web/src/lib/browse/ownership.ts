import { session } from '$lib/stores/session.svelte';

export const NOT_OWNER =
  'Shared by another Immich user. Only the owner can rate, flag, label or tag it.';
export const SOME_NOT_OWNER =
  'Some selected photos are shared by another Immich user. Only the owner can rate, flag, label or tag them.';
export const NOT_OWNER_STACK = 'Shared by another Immich user. Only the owner can stack it.';
export const NONE_OWNED_STACK =
  'The selected photos are shared by another Immich user. Only the owner can stack them.';

export function ownsAsset(asset: { ownerId?: string }): boolean {
  return !asset.ownerId || asset.ownerId === session.user?.id;
}
