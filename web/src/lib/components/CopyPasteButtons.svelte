<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { mdiContentCopy, mdiContentPaste } from '@mdi/js';

  let {
    canCopy,
    canPaste,
    copyTitle,
    pasteTitle,
    onCopy,
    onPaste
  }: {
    canCopy: boolean;
    canPaste: boolean;
    copyTitle: string;
    pasteTitle: string;
    onCopy: () => void;
    onPaste: () => void;
  } = $props();
</script>

<button
  class="flex items-center justify-center py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white/5"
  disabled={!canCopy}
  onclick={onCopy}
  title={copyTitle}
  aria-label="Copy edits"
>
  <Icon path={mdiContentCopy} size={14} />
</button>
<button
  class="flex items-center justify-center py-1.5 px-2 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white/5"
  disabled={!canPaste}
  onclick={onPaste}
  title={pasteTitle}
  aria-label="Paste edits"
>
  <Icon path={mdiContentPaste} size={14} />
</button>
