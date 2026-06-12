import { getJson, sendJson } from './client';

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

export interface JobItem {
  id: string;
  job_id: string;
  asset_id: string;
  status: JobItemStatus;
  error: string | null;
  result: unknown;
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
