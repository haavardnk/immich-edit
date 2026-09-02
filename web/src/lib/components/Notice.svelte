<script lang="ts">
  import { Icon } from '@immich/ui';
  import {
    mdiAlertCircleOutline,
    mdiAlertOutline,
    mdiCheckCircleOutline,
    mdiInformationOutline
  } from '@mdi/js';
  import type { Snippet } from 'svelte';

  type Tone = 'danger' | 'warning' | 'info' | 'success';

  let {
    color = 'danger',
    message,
    class: className,
    children
  }: {
    color?: Tone;
    message: string;
    class?: string;
    children?: Snippet;
  } = $props();

  const icons: Record<Tone, string> = {
    danger: mdiAlertCircleOutline,
    warning: mdiAlertOutline,
    info: mdiInformationOutline,
    success: mdiCheckCircleOutline
  };

  const tones: Record<Tone, string> = {
    danger: 'border-danger-500/25 bg-danger-500/10 text-danger-200',
    warning: 'border-warning-500/25 bg-warning-500/10 text-warning-200',
    info: 'border-info-500/25 bg-info-500/10 text-info-200',
    success: 'border-success-500/25 bg-success-500/10 text-success-200'
  };
</script>

<div
  role={color === 'danger' ? 'alert' : 'status'}
  class="flex items-start gap-2 rounded-lg border px-2.5 py-1.5 text-[11px] leading-snug {tones[
    color
  ]} {className ?? ''}"
>
  <Icon icon={icons[color]} size="14px" class="mt-px flex-none" aria-hidden />
  <div class="flex-1 wrap-anywhere">{message}</div>
  {@render children?.()}
</div>
