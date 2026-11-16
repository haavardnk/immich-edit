<script lang="ts">
  import { goto } from '$app/navigation';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import Popover from '#lib/components/Popover.svelte';
  import ThemeModeChoice from '#lib/components/settings/ThemeModeChoice.svelte';
  import AccountCard from '#lib/components/shell/account/AccountCard.svelte';
  import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
  import { help } from '#lib/commands/app.ts';
  import { settingsHref } from '#lib/settings/pages.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { signedInDevices } from '#lib/stores/signedInDevices.svelte.ts';
  import { Avatar, Button, Icon } from '@immich/ui';
  import { tick } from 'svelte';
  import {
    mdiCogOutline,
    mdiKeyOutline,
    mdiKeyboardOutline,
    mdiLogoutVariant,
    mdiThemeLightDark
  } from '@mdi/js';

  let open = $state(false);

  const user = $derived(session.user);
  const name = $derived(user?.name || user?.email || '?');

  function setOpen(next: boolean): void {
    open = next;
    if (next) void signedInDevices.load();
  }

  function go(href: string): void {
    open = false;
    void goto(href);
  }

  async function shortcuts(): Promise<void> {
    open = false;
    await tick();
    help.run();
  }

  function signOut(): void {
    open = false;
    signOutFlow.request();
  }
</script>

<Popover
  {open}
  onOpenChange={setOpen}
  align="end"
  appearance="panel"
  contentClass="w-80"
  tooltip="Account"
>
  {#snippet trigger(props)}
    <Button
      {...props}
      variant="ghost"
      color="secondary"
      shape="round"
      class="size-10 p-0"
      title=""
      aria-label="Account"
    >
      <Avatar size="small" class="size-8 text-[13px] font-bold" {name} />
    </Button>
  {/snippet}
  {#if user}
    <div class="p-3">
      <AccountCard {user} />
    </div>
  {/if}
  <div class="flex flex-col border-t border-hairline py-1">
    <MenuRow icon={mdiCogOutline} label="Settings" onclick={() => go('/settings')} />
    <div class="flex h-9 items-center gap-2.5 px-3 text-xs">
      <Icon icon={mdiThemeLightDark} size="14px" class="shrink-0 opacity-70" aria-hidden="true" />
      <span class="min-w-0 flex-1">Theme</span>
      <ThemeModeChoice label="Theme" compact />
    </div>
    <MenuRow
      icon={mdiKeyboardOutline}
      label={help.label}
      aria-label={help.label}
      keys={keysFor('help')}
      onclick={() => void shortcuts()}
    />
    <MenuRow
      icon={mdiKeyOutline}
      label="Signed-in devices"
      aria-label="Signed-in devices"
      onclick={() => go(settingsHref('account', 'sessions'))}
    >
      {#if signedInDevices.count !== null}
        <span class="shrink-0 text-[11px] text-dark/65">{signedInDevices.count}</span>
      {/if}
    </MenuRow>
  </div>
  <div class="flex flex-col border-t border-hairline py-1">
    <MenuRow
      icon={mdiLogoutVariant}
      label="Sign out"
      disabled={signOutFlow.signingOut}
      onclick={signOut}
    />
  </div>
</Popover>
