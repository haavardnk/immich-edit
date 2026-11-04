import { browser } from '$app/env';
import { installPlatform, installState, type InstallState } from '#lib/pwa/installState.ts';
import { MediaQuery } from 'svelte/reactivity';

type InstallPromptEvent = Event & {
  prompt: () => Promise<unknown>;
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
};

function isInstallPrompt(event: Event): event is InstallPromptEvent {
  return 'prompt' in event && typeof event.prompt === 'function' && 'userChoice' in event;
}

function homeScreenApp(): boolean {
  return 'standalone' in navigator && navigator.standalone === true;
}

class InstallStore {
  #deferred = $state.raw<InstallPromptEvent | null>(null);
  #installed = $state(false);
  #standalone = new MediaQuery('(display-mode: standalone)');
  #platform = installPlatform(navigator.userAgent, navigator.maxTouchPoints);
  #state: InstallState = $derived(
    installState({
      standalone: this.#standalone.current || homeScreenApp(),
      secure: isSecureContext,
      installed: this.#installed,
      canPrompt: this.#deferred !== null,
      platform: this.#platform
    })
  );

  get state(): InstallState {
    return this.#state;
  }

  capture = (event: Event): void => {
    if (!isInstallPrompt(event)) return;
    event.preventDefault();
    this.#deferred = event;
  };

  markInstalled = (): void => {
    this.#installed = true;
    this.#deferred = null;
  };

  prompt = async (): Promise<void> => {
    const event = this.#deferred;
    if (!event) return;
    this.#deferred = null;
    try {
      await event.prompt();
    } catch {
      this.#deferred = event;
      return;
    }
    const { outcome } = await event.userChoice;
    if (outcome === 'accepted') this.markInstalled();
  };
}

export const install = new InstallStore();

if (browser) {
  window.addEventListener('beforeinstallprompt', install.capture);
  window.addEventListener('appinstalled', install.markInstalled);
}
