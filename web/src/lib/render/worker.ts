import init, { WebRenderer } from '$lib/wasm/web_render';
import { errorMessage } from '$lib/utils/errors';
import { createDispatcher } from './dispatch';
import type { Reply, Request } from './protocol';

const dispatch = createDispatcher(async (canvas) => {
  await init();
  return WebRenderer.create(canvas);
});

self.onmessage = async ({ data }: MessageEvent<Request>) => {
  let reply: Reply;
  try {
    reply = { id: data.id, ok: true, value: await dispatch(data.call) };
  } catch (err) {
    reply = { id: data.id, ok: false, error: errorMessage(err) };
  }
  self.postMessage(reply);
};
