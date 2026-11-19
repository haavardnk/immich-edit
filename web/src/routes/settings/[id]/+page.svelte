<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import SettingsPageView from '#lib/components/settings/SettingsPageView.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { findPage, legacyTarget, settingsHref } from '#lib/settings/pages.ts';
  import { session } from '#lib/stores/session.svelte.ts';

  const id = $derived(page.params.id);
  const entry = $derived(findPage(id));
  const legacy = $derived(entry || !id ? null : legacyTarget(id));
  const shown = $derived(entry !== null && (!entry.adminOnly || session.isAdmin));

  $effect(() => {
    if (legacy) {
      void goto(settingsHref(legacy.page, legacy.anchor), { replaceState: true });
      return;
    }
    if (session.user && !shown) void goto('/settings', { replaceState: true });
  });
</script>

{#if entry && shown}
  {#key entry.id}
    <SettingsPageView {entry} heading={!layout.compact} />
  {/key}
{/if}
