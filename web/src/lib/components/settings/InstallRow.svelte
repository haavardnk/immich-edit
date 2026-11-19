<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import InstallGuide from '#lib/components/settings/InstallGuide.svelte';
  import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
  import type { InstallGuide as Guide } from '#lib/pwa/installState.ts';
  import { install } from '#lib/pwa/install.svelte.ts';
  import { Button, Link } from '@immich/ui';

  const HTTPS_DOCS = 'https://haavardnk.github.io/immich-edit/deploy/#https-for-the-installed-app';

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

{#if current.kind === 'insecure'}
  <SettingsRow label="Install app" anchor="install">
    {#snippet detail()}
      Needs HTTPS. <Link href={HTTPS_DOCS} class="text-primary">How to set up HTTPS</Link>
    {/snippet}
  </SettingsRow>
{:else if current.kind !== 'hidden'}
  <SettingsRow label="Install app" anchor="install">
    {#if current.kind === 'installed'}
      <span class="shrink-0 text-xs text-dark/80 touch:text-sm">Installed</span>
    {:else}
      <Button
        size="small"
        variant="ghost"
        color="secondary"
        class="shrink-0 bg-light-200/60"
        onclick={start}
      >
        Install
      </Button>
    {/if}
  </SettingsRow>
{/if}

{#if guideOpen && current.kind === 'guide'}
  <Dialog title={TITLES[current.guide]} onClose={() => (guideOpen = false)}>
    <InstallGuide guide={current.guide} />
  </Dialog>
{/if}
