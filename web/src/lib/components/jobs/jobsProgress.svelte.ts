import { jobsProgressFraction, jobsProgressText } from '#lib/components/jobs/jobFormat.ts';
import { jobs } from '#lib/stores/jobs.svelte.ts';

class JobsProgress {
  get visible(): boolean {
    return jobs.activeCount > 0;
  }

  get text(): string {
    return jobsProgressText(jobs.active);
  }

  get ariaLabel(): string {
    return `Jobs: ${this.text}`;
  }

  get fraction(): number {
    return jobsProgressFraction(jobs.active);
  }

  readonly toggle = (): void => {
    jobs.toggle();
  };
}

export const jobsProgress = new JobsProgress();
