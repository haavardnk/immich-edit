<script lang="ts">
  import AlbumDetailsForm from '#lib/components/library/AlbumDetailsForm.svelte';
  import { albumActions } from '#lib/stores/albumActions.svelte.ts';
</script>

{#if albumActions.details}
  {#key albumActions.details}
    <AlbumDetailsForm request={albumActions.details} />
  {/key}
{/if}
