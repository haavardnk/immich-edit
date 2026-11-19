import { jobNeedsAttention, type Job } from '#lib/api/jobs.ts';
import { jobKindLabel, jobOutcome } from '#lib/components/jobs/jobFormat.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { readStored, writeStored } from '#lib/utils/storage.ts';

const STORAGE_KEY = 'immich-edit:jobNotifications';
const ICON = '/icon-192.png';

type Persisted = { enabled: boolean };

function currentPermission(): NotificationPermission | null {
  return typeof Notification === 'undefined' ? null : Notification.permission;
}

async function show(title: string, options: NotificationOptions): Promise<void> {
  const registration = await navigator.serviceWorker?.getRegistration().catch(() => undefined);
  if (registration) {
    await registration.showNotification(title, options).catch(() => undefined);
    return;
  }
  try {
    const notification = new Notification(title, options);
    notification.onclick = () => {
      window.focus();
      notification.close();
    };
  } catch {
    return;
  }
}

class JobNotifications {
  #enabled = $state(readStored<Persisted>(STORAGE_KEY)?.enabled === true);
  #permission = $state(currentPermission());

  get supported(): boolean {
    return this.#permission !== null;
  }

  get enabled(): boolean {
    return this.#enabled && this.#permission === 'granted';
  }

  get blocked(): boolean {
    return this.#permission === 'denied';
  }

  toggle = async (): Promise<void> => {
    if (this.enabled) {
      this.#store(false);
      return;
    }
    if (typeof Notification === 'undefined') return;
    const answer =
      Notification.permission === 'default'
        ? await Notification.requestPermission()
        : Notification.permission;
    this.#permission = answer;
    if (answer === 'granted') {
      this.#store(true);
      return;
    }
    toasts.push('warn', 'Notifications are blocked for this site', {
      description: 'Allow them in the browser settings, then turn this on again.'
    });
  };

  notify(job: Job): void {
    if (!this.#enabled || currentPermission() !== 'granted') return;
    if (job.status === 'cancelled' || document.visibilityState !== 'hidden') return;
    const label = jobKindLabel(job.kind);
    void show(jobNeedsAttention(job) ? `${label} finished with errors` : `${label} finished`, {
      body: jobOutcome(job),
      tag: `job-${job.id}`,
      icon: ICON
    });
  }

  #store(enabled: boolean): void {
    this.#enabled = enabled;
    writeStored(STORAGE_KEY, { enabled } satisfies Persisted);
  }
}

export const jobNotifications = new JobNotifications();
