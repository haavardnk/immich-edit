<script lang="ts">
  import { runEditorRoute } from '#lib/editor/editorRoute.svelte.ts';

  let id = $state('a1');

  export function show(next: string): void {
    id = next;
  }

  runEditorRoute(() => id);
</script>
