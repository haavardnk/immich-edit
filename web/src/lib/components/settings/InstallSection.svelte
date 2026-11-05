<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import Notice from '#lib/components/Notice.svelte';
  import InstallGuide from '#lib/components/settings/InstallGuide.svelte';
  import type { InstallGuide as Guide, InstallState } from '#lib/pwa/installState.ts';
  import { install } from '#lib/pwa/install.svelte.ts';
  import { Button } from '@immich/ui';

  const HTTPS_DOCS = 'https://haavardnk.github.io/immich-edit/deploy/#https-for-the-installed-app';

  const STATUS: Record<Exclude<InstallState['kind'], 'hidden'>, string> = {
    insecure: 'Unavailable on this connection.',
    installed: 'Installed. Open immich-edit from your Home Screen, Dock or app list.',
    prompt: 'Not installed. Your browser installs it in one step.',
    guide: 'Your browser adds it in a few steps.'
  };

  const TITLES: Record<Guide, string> = {
    ios: 'Add immich-edit to your Home Screen',
    'safari-mac': 'Add immich-edit to your Dock',
    browser: 'Install immich-edit'
  };

  let guideOpen = $state(false);

  const current = $derived(install.state);

  function start(): void {
    if (current.kind === 'prompt') void install.prompt();
    else guideOpen = true;
  }
</script>

<section class="space-y-2 py-2">
  {#if current.kind === 'prompt' || current.kind === 'guide'}
    <Button size="small" onclick={start}>Install app</Button>
  {/if}
  {#if current.kind !== 'hidden'}
    <p class="text-xs text-dark/65" aria-live="polite">{STATUS[current.kind]}</p>
  {/if}
  {#if current.kind === 'insecure'}
    <Notice
      color="warning"
      message="Installing needs HTTPS. This server is reached over {location.origin}, so the browser blocks installing and offline loading. Editing still works."
    />
    <a class="text-xs text-primary hover:underline" href={HTTPS_DOCS} target="_blank" rel="noopener"
      >How to set up HTTPS</a
    >
  {/if}
</section>

{#if guideOpen && current.kind === 'guide'}
  <Dialog title={TITLES[current.guide]} onClose={() => (guideOpen = false)}>
    <InstallGuide guide={current.guide} />
  </Dialog>
{/if}
