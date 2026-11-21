<script lang="ts">
  import TagDetailsForm from '#lib/components/library/TagDetailsForm.svelte';
  import { tagActions } from '#lib/stores/tagActions.svelte.ts';
</script>

{#if tagActions.details}
  {#key tagActions.details}
    <TagDetailsForm request={tagActions.details} />
  {/key}
{/if}
