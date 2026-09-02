<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { session } from '$lib/stores/session.svelte';
  import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
  import AccountSection from '$lib/components/settings/AccountSection.svelte';
  import SessionsSection from '$lib/components/settings/SessionsSection.svelte';
  import UsersSection from '$lib/components/settings/UsersSection.svelte';
  import MaskModelsSection from '$lib/components/settings/MaskModelsSection.svelte';
  import InstanceSection from '$lib/components/settings/InstanceSection.svelte';
  import { mdiAccountCircleOutline, mdiCogOutline } from '@mdi/js';

  type AdminSection = 'users' | 'models' | 'instance';

  let appOpen = $state(false);
  let accountOpen = $state(false);
  let sessionsOpen = $state(false);
  let adminSection = $state<AdminSection | null>(null);

  const requestedAdminSection = $derived(adminSectionFromHash(page.url.hash));

  function adminSectionFromHash(hash: string): AdminSection | null {
    if (hash === '#users') return 'users';
    if (hash === '#models') return 'models';
    if (hash === '#instance') return 'instance';
    return null;
  }

  function setAdminSection(section: AdminSection, open: boolean): void {
    if (!open) {
      if (adminSection === section) adminSection = null;
      return;
    }
    adminSection = section;
    void goto(`/settings#${section}`, { replaceState: true, noScroll: true, keepFocus: true });
  }

  $effect(() => {
    const section = requestedAdminSection;
    if (section) {
      adminSection = section;
      appOpen = true;
      accountOpen = false;
      return;
    }
    if (page.url.hash === '') {
      appOpen = false;
      accountOpen = false;
      sessionsOpen = false;
      return;
    }
    if (page.url.hash === '#account' || page.url.hash === '#sessions') {
      appOpen = false;
      accountOpen = true;
      sessionsOpen = page.url.hash === '#sessions';
    }
  });
</script>

{#if session.user}
  <div class="space-y-4">
    {#if session.isAdmin}
      <SettingsGroup
        title="App settings"
        description="Manage access, local models, and the connected Immich server."
        icon={mdiCogOutline}
        open={appOpen}
        onOpenChange={(open) => (appOpen = open)}
      >
        <div class="space-y-3">
          <SettingsGroup
            title="Users"
            description="Manage access to this immich-edit installation."
            open={adminSection === 'users'}
            onOpenChange={(open) => setAdminSection('users', open)}
          >
            <UsersSection />
          </SettingsGroup>
          <SettingsGroup
            title="Mask models"
            description="Manage models used for AI-assisted masks."
            open={adminSection === 'models'}
            onOpenChange={(open) => setAdminSection('models', open)}
          >
            <MaskModelsSection />
          </SettingsGroup>
          <SettingsGroup
            title="Immich instance"
            description="Manage the connected Immich server."
            open={adminSection === 'instance'}
            onOpenChange={(open) => setAdminSection('instance', open)}
          >
            <InstanceSection />
          </SettingsGroup>
        </div>
      </SettingsGroup>
    {/if}

    <SettingsGroup
      title="Account"
      description="Review your identity, sign-in method, and active sessions."
      icon={mdiAccountCircleOutline}
      open={accountOpen}
      onOpenChange={(open) => (accountOpen = open)}
    >
      <AccountSection />
      <SettingsGroup
        title="Signed-in devices"
        description="Review and revoke active sessions."
        open={sessionsOpen}
        onOpenChange={(open) => (sessionsOpen = open)}
      >
        <SessionsSection />
      </SettingsGroup>
    </SettingsGroup>
  </div>
{/if}
