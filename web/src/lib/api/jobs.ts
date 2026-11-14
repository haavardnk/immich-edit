import { getJson, sendJson, url } from './client';
import {
  finishParams,
  type BitDepthOpt,
  type ColorSpaceOpt,
  type ExportFormat,
  type ExportMetadata,
  type ExportOptions,
  type FinishParams,
  type ImmichExportOptions,
  type PngCompressionOpt,
  type TiffCompressionOpt
} from './export';
import type { EditManifest } from '#lib/types/edits.ts';
import type { CopySelection } from '#lib/editor/copyTools.ts';

export const JOB_KINDS = {
  exportImmich: 'export_immich',
  downloadZip: 'download_zip',
  applyPreset: 'apply_preset',
  pasteEdits: 'paste_edits',
  resetEdits: 'reset_edits'
} as const;

export const TRANSFER_JOB_KINDS: ReadonlySet<string> = new Set([
  JOB_KINDS.exportImmich,
  JOB_KINDS.downloadZip
]);

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
export type JobItemStatus = 'pending' | 'running' | 'completed' | 'failed';

export interface Job {
  id: string;
  kind: string;
  status: JobStatus;
  target: unknown;
  params: unknown;
  total: number;
  completed: number;
  failed: number;
  cancelled_at: string | null;
  created_at: string;
  updated_at: string;
  eta_secs?: number | null;
  item_ms?: number | null;
}

export interface JobItemResult {
  filename?: string;
  warnings?: string[];
  original_trashed?: boolean;
}

export interface JobItem {
  id: string;
  job_id: string;
  asset_id: string;
  status: JobItemStatus;
  error: string | null;
  result: JobItemResult | null;
  idempotency_key: string | null;
  attempts: number;
  created_at: string;
  updated_at: string;
}

export interface JobDetail {
  job: Job;
  items: JobItem[];
}

export function isJobActive(status: JobStatus): boolean {
  return status === 'pending' || status === 'running';
}

export function splitJobs(list: readonly Job[]): { running: Job[]; finished: Job[] } {
  return {
    running: list.filter((job) => isJobActive(job.status)),
    finished: list.filter((job) => !isJobActive(job.status))
  };
}

export function jobNeedsAttention(job: Job): boolean {
  return !isJobActive(job.status) && (job.failed > 0 || job.status === 'failed');
}

export function zipReady(job: Job): boolean {
  return job.kind === JOB_KINDS.downloadZip && job.status === 'completed' && job.completed > 0;
}

export function listJobs(): Promise<Job[]> {
  return getJson('/api/jobs');
}

export function getJob(id: string): Promise<JobDetail> {
  return getJson(url`/api/jobs/${id}`);
}

export function cancelJob(id: string): Promise<void> {
  return sendJson('POST', url`/api/jobs/${id}/cancel`, undefined);
}

export function clearJobs(): Promise<void> {
  return sendJson('DELETE', '/api/jobs', undefined);
}

export function jobDownloadUrl(id: string): string {
  return url`/api/jobs/${id}/download`;
}

interface ExportJobParams extends FinishParams {
  format: ExportFormat;
  quality: number;
  metadata: ExportMetadata;
  bit_depth: BitDepthOpt;
  png_compression: PngCompressionOpt;
  tiff_compression: TiffCompressionOpt;
  lossless: boolean;
  color_space: ColorSpaceOpt;
  filename_template: string;
}

function baseParams(opts: ExportOptions): ExportJobParams {
  return {
    format: opts.format,
    quality: opts.quality,
    metadata: opts.metadata,
    bit_depth: opts.bitDepth,
    png_compression: opts.pngCompression,
    tiff_compression: opts.tiffCompression,
    lossless: opts.lossless,
    color_space: opts.colorSpace,
    filename_template: opts.filenameTemplate,
    ...finishParams(opts)
  };
}

export function createImmichExportJob(assetIds: string[], opts: ImmichExportOptions): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: JOB_KINDS.exportImmich,
    asset_ids: assetIds,
    params: {
      ...baseParams(opts),
      album_ids: opts.albumIds,
      tag_ids: opts.tagIds,
      favorite: opts.favorite,
      stack_with_original: opts.stackWithOriginal,
      stack_primary: opts.stackPrimary,
      replace_original: opts.replaceOriginal
    }
  });
}

export function createZipExportJob(assetIds: string[], opts: ExportOptions): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: JOB_KINDS.downloadZip,
    asset_ids: assetIds,
    params: baseParams(opts)
  });
}

export interface ApplyPresetOptions {
  includeGeometry: boolean;
  includeMasks: boolean;
  amount: number;
}

export function createApplyPresetJob(
  assetIds: string[],
  presetId: string,
  opts: ApplyPresetOptions
): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: JOB_KINDS.applyPreset,
    asset_ids: assetIds,
    params: {
      preset_id: presetId,
      include_geometry: opts.includeGeometry,
      include_masks: opts.includeMasks,
      amount: opts.amount
    }
  });
}

export function createPasteEditsJob(
  assetIds: string[],
  manifest: EditManifest,
  tools: CopySelection
): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: JOB_KINDS.pasteEdits,
    asset_ids: assetIds,
    params: { manifest, tools }
  });
}

export function createResetEditsJob(assetIds: string[]): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: JOB_KINDS.resetEdits,
    asset_ids: assetIds,
    params: {}
  });
}
