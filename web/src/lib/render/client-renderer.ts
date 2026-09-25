import type { Roi } from '$lib/api/preview';
import { fetchRaster } from '$lib/api/rasters';
import { fetchDcpBytes, fetchLutCube, fetchSource } from '$lib/api/source';
import type { Edits } from '$lib/types/edits';
import { RenderHost } from './host';
import type { RenderInputs, RenderView, RenderedFrame, SourceInfo } from './protocol';

export class ClientRenderer {
  private readonly host: RenderHost;
  readonly initMs: number;
  private dcpId: string | null | undefined = undefined;
  private lutId: string | null = null;
  private readonly rasters = new Set<string>();

  private constructor(host: RenderHost, initMs: number) {
    this.host = host;
    this.initMs = initMs;
  }

  static async start(): Promise<ClientRenderer> {
    if (!isSecureContext) throw new Error('browsers only offer WebGPU over HTTPS or on localhost');
    if (!('gpu' in navigator) || !navigator.gpu) throw new Error('this browser has no WebGPU');
    const started = performance.now();
    const host = await RenderHost.start();
    return new ClientRenderer(host, performance.now() - started);
  }

  get adapter(): string {
    return this.host.adapter;
  }

  inputs(edits: Edits, maxEdge: number): Promise<RenderInputs> {
    return this.host.inputs(edits, maxEdge);
  }

  async loadSource(
    assetId: string,
    edits: Edits,
    maxEdge: number,
    signal: AbortSignal
  ): Promise<SourceInfo> {
    const { bytes, dcpId } = await fetchSource(assetId, edits, maxEdge, signal);
    const changed = dcpId !== this.dcpId;
    const dcp = changed && dcpId ? await fetchDcpBytes(dcpId) : null;
    signal.throwIfAborted();
    const setDcp = changed ? this.host.setDcp(dcp) : Promise.resolve();
    const info = this.host.setSource(bytes);
    await setDcp;
    this.dcpId = dcpId;
    return info;
  }

  async loadTile(
    assetId: string,
    edits: Edits,
    maxEdge: number,
    roi: Roi,
    signal: AbortSignal
  ): Promise<SourceInfo> {
    const { bytes } = await fetchSource(assetId, edits, maxEdge, signal, roi);
    signal.throwIfAborted();
    return this.host.setTile(bytes);
  }

  async prepare(inputs: RenderInputs): Promise<void> {
    await Promise.all([this.prepareLut(inputs.lut), this.prepareRasters(inputs.rasters)]);
  }

  render(edits: Edits, view: RenderView): Promise<RenderedFrame> {
    return this.host.render(edits, view);
  }

  dispose(): void {
    this.host.dispose();
  }

  private async prepareLut(id: string | null): Promise<void> {
    if (id === this.lutId) return;
    const previous = this.lutId;
    if (id) await this.host.setLut(id, await fetchLutCube(id));
    this.lutId = id;
    if (previous) await this.host.dropLut(previous);
  }

  private async prepareRasters(ids: string[]): Promise<void> {
    const wanted = new Set(ids);
    const stale = [...this.rasters].filter((id) => !wanted.has(id));
    for (const id of stale) this.rasters.delete(id);
    await Promise.all([
      ...stale.map((id) => this.host.dropRaster(id)),
      ...ids
        .filter((id) => !this.rasters.has(id))
        .map(async (id) => {
          const raster = await fetchRaster(id);
          await this.host.setRaster(id, raster.width, raster.height, raster.bytes.buffer);
          this.rasters.add(id);
        })
    ]);
  }
}
