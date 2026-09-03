import { getEdits } from '$lib/api/edits';
import { getFaces, type FaceBox } from '$lib/api/faces';
import { manifestToEdits } from '$lib/edits/manifest';
import { viewTransform, type ViewTransform } from '$lib/utils/canvasCoords';

export interface FaceData {
  faces: FaceBox[];
  savedView: ViewTransform | null;
}

const MAX_ENTRIES = 50;

const EMPTY: FaceData = { faces: [], savedView: null };

const cache = new Map<string, FaceData>();
const inflight = new Map<string, Promise<FaceData>>();

function put(id: string, data: FaceData): void {
  cache.delete(id);
  cache.set(id, data);
  while (cache.size > MAX_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest === undefined) break;
    cache.delete(oldest);
  }
}

async function fetchFaceData(id: string): Promise<FaceData> {
  const faces = await getFaces(id);
  const first = faces[0];
  if (!first) return EMPTY;
  const record = await getEdits(id);
  const savedView = viewTransform(manifestToEdits(record.manifest), {
    source_w: first.source_w,
    source_h: first.source_h
  });
  return { faces, savedView };
}

export function cachedFaceData(id: string): FaceData | null {
  return cache.get(id) ?? null;
}

export function loadFaceData(id: string): Promise<FaceData> {
  const cached = cache.get(id);
  if (cached) return Promise.resolve(cached);
  const pending = inflight.get(id);
  if (pending) return pending;
  const promise = fetchFaceData(id)
    .then((data) => {
      put(id, data);
      return data;
    })
    .catch(() => EMPTY)
    .finally(() => {
      inflight.delete(id);
    });
  inflight.set(id, promise);
  return promise;
}

export function invalidateFaceData(id: string): void {
  cache.delete(id);
  inflight.delete(id);
}

if (typeof window !== 'undefined') {
  const drop = (ev: Event): void => {
    invalidateFaceData((ev as CustomEvent<{ id: string }>).detail.id);
  };
  window.addEventListener('immich-edit:edits-saved', drop);
  window.addEventListener('immich-edit:edits-deleted', drop);
}
