<script lang="ts">
  import AccentPicker from '#lib/components/settings/AccentPicker.svelte';
  import ThemeModeChoice from '#lib/components/settings/ThemeModeChoice.svelte';
  import ThemePicker from '#lib/components/settings/ThemePicker.svelte';
</script>

<div class="space-y-5 py-2">
  <section class="space-y-2">
    <h2 class="text-xs font-medium">Mode</h2>
    <ThemeModeChoice />
  </section>
  <div class="space-y-4">
    <ThemePicker scheme="light" label="Light theme" />
    <AccentPicker scheme="light" label="Light accent" />
  </div>
  <div class="space-y-4">
    <ThemePicker scheme="dark" label="Dark theme" />
    <AccentPicker scheme="dark" label="Dark accent" />
  </div>
</div>
