<script lang="ts">
	import '../app.css';
  import Shell from '$lib/components/shell/Shell.svelte';

  let { children } = $props();
</script>

<Shell>
  {@render children()}
</Shell>
