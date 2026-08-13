<script lang="ts">
  import { onMount } from 'svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { session } from '$lib/stores/session.svelte';
  import AccountSection from '$lib/components/settings/AccountSection.svelte';
  import SessionsSection from '$lib/components/settings/SessionsSection.svelte';
  import UsersSection from '$lib/components/settings/UsersSection.svelte';
  import MaskModelsSection from '$lib/components/settings/MaskModelsSection.svelte';
  import InstanceSection from '$lib/components/settings/InstanceSection.svelte';

  onMount(() => {
    editor.unload();
  });
</script>

{#if session.user}
  <AccountSection />
  <SessionsSection />
  {#if session.isAdmin}
    <UsersSection />
    <MaskModelsSection />
    <InstanceSection />
  {/if}
{/if}
