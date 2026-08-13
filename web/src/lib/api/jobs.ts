import { getJson, sendJson } from './client';
import type {
  BitDepthOpt,
  ExportFormat,
  ExportOptions,
  ImmichExportOptions,
  PngCompressionOpt,
  TiffCompressionOpt
} from './export';
import type { EditManifest } from '$lib/types/edits';
import type { SearchQuery } from '$lib/types/search';
import type { CopySections } from '$lib/copyPaste';

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
}

export interface JobItemResult {
  filename?: string;
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

export function listJobs(): Promise<Job[]> {
  return getJson('/api/jobs');
}

export function getJob(id: string): Promise<JobDetail> {
  return getJson(`/api/jobs/${id}`);
}

export function cancelJob(id: string): Promise<void> {
  return sendJson('POST', `/api/jobs/${id}/cancel`, undefined);
}

export function clearJobs(): Promise<void> {
  return sendJson('DELETE', '/api/jobs', undefined);
}

export function jobDownloadUrl(id: string): string {
  return `/api/jobs/${id}/download`;
}

interface ExportJobParams {
  format: ExportFormat;
  quality: number;
  include_exif: boolean;
  bit_depth: BitDepthOpt;
  png_compression: PngCompressionOpt;
  tiff_compression: TiffCompressionOpt;
  lossless: boolean;
}

function baseParams(opts: ExportOptions): ExportJobParams {
  return {
    format: opts.format,
    quality: opts.quality,
    include_exif: opts.includeExif,
    bit_depth: opts.bitDepth,
    png_compression: opts.pngCompression,
    tiff_compression: opts.tiffCompression,
    lossless: opts.lossless
  };
}

export type JobTarget = { assetIds: string[] } | { search: SearchQuery };

type JobTargetFields = { asset_ids: string[] } | { target: { search: SearchQuery } };

function targetFields(target: JobTarget): JobTargetFields {
  if ('assetIds' in target) return { asset_ids: target.assetIds };
  return { target: { search: target.search } };
}

export function createImmichExportJob(target: JobTarget, opts: ImmichExportOptions): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: 'export_immich',
    ...targetFields(target),
    params: {
      ...baseParams(opts),
      album_ids: opts.albumIds,
      tag_ids: opts.tagIds,
      favorite: opts.favorite,
      stack_with_original: opts.stackWithOriginal,
      stack_primary: opts.stackPrimary,
      filename_suffix: opts.filenameSuffix
    }
  });
}

export function createZipExportJob(
  target: JobTarget,
  opts: ExportOptions,
  filenameSuffix: string
): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: 'download_zip',
    ...targetFields(target),
    params: { ...baseParams(opts), filename_suffix: filenameSuffix }
  });
}

export interface ApplyPresetOptions {
  includeGeometry: boolean;
  includeMasks: boolean;
}

export function createApplyPresetJob(
  target: JobTarget,
  presetId: string,
  opts: ApplyPresetOptions
): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: 'apply_preset',
    ...targetFields(target),
    params: {
      preset_id: presetId,
      include_geometry: opts.includeGeometry,
      include_masks: opts.includeMasks
    }
  });
}

export function createPasteEditsJob(
  target: JobTarget,
  manifest: EditManifest,
  sections: CopySections
): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: 'paste_edits',
    ...targetFields(target),
    params: { manifest, sections }
  });
}

export function createResetEditsJob(target: JobTarget): Promise<Job> {
  return sendJson('POST', '/api/jobs', {
    kind: 'reset_edits',
    ...targetFields(target),
    params: {}
  });
}
