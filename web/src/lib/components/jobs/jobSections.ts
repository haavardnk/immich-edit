import { splitJobs, type Job } from '#lib/api/jobs.ts';

export type JobSection = { label: string | null; jobs: readonly Job[] };

export function jobSections(list: readonly Job[], grouped: boolean): JobSection[] {
  if (!grouped) return list.length > 0 ? [{ label: null, jobs: list }] : [];
  const { running, finished } = splitJobs(list);
  return [
    { label: 'In progress', jobs: running },
    { label: 'Finished', jobs: finished }
  ].filter((section) => section.jobs.length > 0);
}
