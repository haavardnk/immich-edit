<script lang="ts">
  import SettingsPagesNav from '#lib/components/settings/SettingsPagesNav.svelte';
  import SettingsSearch from '#lib/components/settings/SettingsSearch.svelte';
</script>

<aside
  aria-label="Settings"
  class="flex h-full min-h-0 w-64 flex-col bg-light-50 text-dark wide:w-full"
>
  <div
    class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto pt-3 pe-4 pb-4 ps-0 scrollbar-hidden wide:px-2.5"
  >
    <div
      class="ps-5 pt-2 text-[10px] font-semibold tracking-wide text-dark/65 uppercase wide:hidden"
    >
      Settings
    </div>
    <div class="flex flex-col gap-3 ps-3 wide:ps-0">
      <SettingsSearch>
        <SettingsPagesNav />
      </SettingsSearch>
    </div>
  </div>
</aside>
