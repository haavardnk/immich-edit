import {
  VIEWER_CHANNEL,
  viewerMessage,
  type ViewerMessage,
  type ViewerRender,
  type ViewerView
} from '#lib/viewer/protocol.ts';

export class ViewerSync {
  render = $state.raw<ViewerRender | null>(null);
  view = $state.raw<ViewerView | null>(null);
  linked = $state(false);
  revision = $state(0);
  private last = '';

  connect(): () => void {
    if (typeof BroadcastChannel === 'undefined') return () => {};
    const channel = new BroadcastChannel(VIEWER_CHANNEL);
    const post = (message: ViewerMessage): void => channel.postMessage(message);
    const bye = (): void => post({ kind: 'viewer-bye' });
    channel.onmessage = (event) => {
      const message = viewerMessage(event.data);
      if (message?.kind === 'editor-hello') post({ kind: 'viewer-hello' });
      if (message) this.receive(message);
    };
    post({ kind: 'viewer-hello' });
    window.addEventListener('pagehide', bye);
    return () => {
      window.removeEventListener('pagehide', bye);
      bye();
      channel.close();
    };
  }

  receive(message: ViewerMessage): void {
    if (message.kind === 'editor-bye') this.linked = false;
    else if (message.kind === 'view') this.receiveView(message.view);
    else if (message.kind === 'render') this.receiveRender(message.render);
  }

  private receiveView(view: ViewerView): void {
    this.view = view;
    this.linked = true;
  }

  private receiveRender(render: ViewerRender): void {
    this.linked = true;
    const text = JSON.stringify(render);
    if (text === this.last) return;
    this.last = text;
    this.render = render;
    this.revision += 1;
  }
}
