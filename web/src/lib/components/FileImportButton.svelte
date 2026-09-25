<script lang="ts">
  import { Button } from '@immich/ui';
  import { mdiUpload } from '@mdi/js';

  let {
    accept,
    label,
    fullWidth = false,
    class: className = '',
    onfile
  }: {
    accept: string;
    label: string;
    fullWidth?: boolean;
    class?: string;
    onfile: (file: File) => Promise<void>;
  } = $props();

  let input = $state<HTMLInputElement | null>(null);
  let busy = $state(false);

  async function pick(e: Event): Promise<void> {
    const target = e.currentTarget as HTMLInputElement;
    const file = target.files?.[0];
    target.value = '';
    if (!file) return;
    busy = true;
    try {
      await onfile(file);
    } finally {
      busy = false;
    }
  }
</script>

<input bind:this={input} type="file" {accept} class="hidden" onchange={(e) => void pick(e)} />
<Button
  type="button"
  size="tiny"
  variant="ghost"
  color="secondary"
  {fullWidth}
  class="panel-action h-7 {className}"
  leadingIcon={mdiUpload}
  disabled={busy}
  onclick={() => input?.click()}
>
  {busy ? 'Importing…' : label}
</Button>
