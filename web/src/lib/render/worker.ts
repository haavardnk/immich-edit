import init, { WebRenderer, render_inputs } from '$lib/wasm/web_render';
import { errorMessage } from '$lib/utils/errors';
import { createDispatcher } from './dispatch';
import type { Reply, Request } from './protocol';

const dispatch = createDispatcher(async () => {
  await init();
  return { renderer: await WebRenderer.create(), inputs: render_inputs };
});

self.onmessage = async ({ data }: MessageEvent<Request>) => {
  try {
    const { value, transfer } = await dispatch(data.call);
    self.postMessage({ id: data.id, ok: true, value } satisfies Reply, { transfer });
  } catch (err) {
    self.postMessage({ id: data.id, ok: false, error: errorMessage(err) } satisfies Reply);
  }
};
