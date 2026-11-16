<script lang="ts">
  import InstanceRow from '#lib/components/settings/InstanceRow.svelte';
  import LensDefaultRow from '#lib/components/settings/LensDefaultRow.svelte';
  import ModelsGroup from '#lib/components/settings/ModelsGroup.svelte';
  import SettingsGroup from '#lib/components/settings/SettingsGroup.svelte';
  import UsersGroup from '#lib/components/settings/UsersGroup.svelte';
</script>

<div class="flex flex-col gap-7 touch:gap-5">
  <SettingsGroup title="Server" anchor="server-settings">
    <InstanceRow />
    <LensDefaultRow />
  </SettingsGroup>
  <UsersGroup />
  <ModelsGroup />
</div>
