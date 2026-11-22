import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url';

type MapLibre = typeof import('maplibre-gl');

let loading: Promise<MapLibre> | null = null;

export function loadMapLibre(): Promise<MapLibre> {
  loading ??= Promise.all([import('maplibre-gl'), import('maplibre-gl/dist/maplibre-gl.css')])
    .then(([lib]) => {
      lib.setWorkerUrl(workerUrl);
      return lib;
    })
    .catch((error: unknown) => {
      loading = null;
      throw error;
    });
  return loading;
}
