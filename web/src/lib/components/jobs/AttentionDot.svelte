<script lang="ts">
  let { class: className = '' }: { class?: string } = $props();
</script>

<span
  class="pointer-events-none size-2.5 rounded-full bg-danger {className}"
  data-testid="jobs-attention"
  aria-hidden="true"
></span>
