<script lang="ts">
  import { goto } from '$app/navigation';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import AttentionDot from '#lib/components/jobs/AttentionDot.svelte';
  import SheetScrim from '#lib/components/sheet/SheetScrim.svelte';
  import AccountBadges from '#lib/components/settings/AccountBadges.svelte';
  import ThemeModeChoice from '#lib/components/settings/ThemeModeChoice.svelte';
  import ImmichServerLine from '#lib/components/shell/touch/ImmichServerLine.svelte';
  import { accountDrawer } from '#lib/components/shell/touch/accountDrawer.svelte.ts';
  import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
  import { help } from '#lib/commands/app.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { gestures } from '#lib/gestures/gestures.ts';
  import { swipeRecognizer } from '#lib/gestures/swipe.ts';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { holdModalSheet } from '#lib/stores/overlays.ts';
  import { session } from '#lib/stores/session.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Avatar, Button, IconButton, LoadingSpinner } from '@immich/ui';
  import {
    mdiClose,
    mdiCogOutline,
    mdiFormatListChecks,
    mdiKeyboardOutline,
    mdiLogoutVariant
  } from '@mdi/js';
  import { Dialog } from 'bits-ui';
  import { tick } from 'svelte';
  import type { Attachment } from 'svelte/attachments';

  let panel: HTMLElement | null = null;

  const user = $derived(session.user);

  const bindPanel: Attachment<HTMLElement> = (element) => {
    panel = element;
    return () => {
      if (panel === element) panel = null;
    };
  };

  function follow(dx: number): void {
    if (panel) panel.style.translate = `${Math.min(0, dx)}px 0`;
  }

  function settle(): void {
    if (panel) panel.style.translate = '';
  }

  const swipeClose = gestures({
    touchAction: 'pan-y',
    recognizers: [
      swipeRecognizer({
        directions: ['left'],
        move: ({ dx }) => follow(dx),
        end: ({ direction }) => {
          if (direction === 'left') accountDrawer.close();
          else settle();
        },
        cancel: settle
      })
    ]
  });

  function openSettings(): void {
    accountDrawer.close();
    void goto('/settings');
  }

  async function closeDrawer(): Promise<void> {
    accountDrawer.close();
    await tick();
  }

  async function openJobs(): Promise<void> {
    await closeDrawer();
    if (!jobs.open) jobs.toggle();
  }

  async function openShortcuts(): Promise<void> {
    await closeDrawer();
    help.run();
  }

  async function signOut(): Promise<void> {
    await closeDrawer();
    signOutFlow.request();
  }

  const jobsLabel = $derived(
    [
      'Jobs',
      jobs.activeCount > 0 ? `${countOf(jobs.activeCount, 'job')} running` : null,
      jobs.attention ? 'a job finished with failures' : null
    ]
      .filter(Boolean)
      .join(', ')
  );

  $effect(() => {
    if (!accountDrawer.open) return;
    return holdModalSheet();
  });
</script>

<Dialog.Root
  bind:open={
    () => accountDrawer.open,
    (open) => {
      if (open) accountDrawer.show();
      else accountDrawer.close();
    }
  }
>
  <Dialog.Portal>
    <SheetScrim />
    <Dialog.Content>
      {#snippet child({ props })}
        <div
          {...props}
          class="fixed inset-y-0 left-0 z-50 flex w-80 max-w-[85vw] flex-col overflow-y-auto rounded-e-3xl bg-light-50 pt-[calc(var(--safe-top)+16px)] pr-3 pb-[calc(var(--safe-bottom)+16px)] pl-[calc(var(--safe-left)+12px)] text-dark shadow-popover outline-none starting:-translate-x-full motion-safe:transition-[translate] motion-safe:duration-300 motion-safe:ease-out"
          {@attach bindPanel}
          {@attach swipeClose}
        >
          <div class="mb-2 flex items-center justify-between ps-2">
            <Dialog.Title class="text-sm font-semibold text-dark/65">Account</Dialog.Title>
            <IconButton
              size="medium"
              variant="ghost"
              color="secondary"
              shape="round"
              class="size-11"
              icon={mdiClose}
              aria-label="Close account menu"
              onclick={() => void closeDrawer()}
            />
          </div>
          {#if user}
            <div class="flex items-center gap-3 rounded-2xl bg-light-100 p-3.5">
              <Avatar size="medium" name={user.name || user.email} />
              <div class="min-w-0 flex-1">
                <div class="truncate text-base font-semibold text-light-950">
                  {user.name || user.email}
                </div>
                <div class="truncate text-sm text-dark/65">{user.email}</div>
                <div class="mt-1.5 flex flex-wrap gap-1.5">
                  <AccountBadges {user} />
                </div>
              </div>
            </div>
            <ImmichServerLine class="truncate px-3.5 pt-2.5 pb-1.5 text-[12.5px] text-dark/65" />
          {/if}
          <div class="mt-3 flex flex-col">
            <MenuRow icon={mdiCogOutline} label="Settings" onclick={openSettings} />
            <MenuRow
              icon={mdiFormatListChecks}
              label="Jobs"
              aria-label={jobsLabel}
              onclick={openJobs}
            >
              {#if jobs.activeCount > 0}
                <span class="flex shrink-0 items-center gap-1.5 text-sm text-dark/65">
                  <LoadingSpinner size="small" />
                  {jobs.activeCount}
                </span>
              {/if}
              {#if jobs.attention}
                <AttentionDot class="shrink-0" />
              {/if}
            </MenuRow>
            {#if layout.hasKeyboard}
              <MenuRow
                icon={mdiKeyboardOutline}
                label={help.label}
                keys={keysFor('help')}
                onclick={openShortcuts}
              />
            {/if}
          </div>
          <div class="my-2 h-px bg-hairline"></div>
          <div class="flex flex-col gap-2 px-1 py-1">
            <span class="text-xs font-medium text-dark/65">Theme</span>
            <ThemeModeChoice label="Theme" />
          </div>
          <div class="my-2 h-px bg-hairline"></div>
          <Button
            variant="ghost"
            color="danger"
            class="min-h-13 w-full justify-start gap-3.5 px-4 text-base font-normal"
            leadingIcon={mdiLogoutVariant}
            loading={signOutFlow.signingOut}
            disabled={signOutFlow.signingOut}
            onclick={signOut}
          >
            Sign out
          </Button>
        </div>
      {/snippet}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
