<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import SettingsPagesNav from '#lib/components/settings/SettingsPagesNav.svelte';
  import SettingsSearch from '#lib/components/settings/SettingsSearch.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { legacyTarget, settingsHref } from '#lib/settings/pages.ts';
  import { session } from '#lib/stores/session.svelte.ts';

  const legacy = $derived(legacyTarget(page.url.hash.slice(1)));
  const target = $derived(
    legacy
      ? settingsHref(legacy.page, legacy.anchor)
      : layout.compact
        ? null
        : settingsHref('preferences')
  );

  $effect(() => {
    if (target) void goto(target, { replaceState: true });
  });
</script>

{#if session.user && !target}
  <div class="flex flex-col gap-3">
    <SettingsSearch>
      <SettingsPagesNav />
    </SettingsSearch>
  </div>
{/if}
