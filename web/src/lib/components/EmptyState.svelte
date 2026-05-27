<script lang="ts">
  interface Props {
    title: string;
    message?: string;
  }
  let { title, message }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 px-6 text-center text-immich-dark-fg/50">
  <h3 class="text-sm font-medium text-immich-dark-fg/70">{title}</h3>
  {#if message}
    <p class="text-xs mt-1 max-w-xs">{message}</p>
  {/if}
</div>
