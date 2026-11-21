<script lang="ts">
  import { Textarea, type TextareaProps } from '@immich/ui';

  let {
    ref = $bindable(null),
    value = $bindable(''),
    class: className,
    ...props
  }: TextareaProps = $props();
</script>

<Textarea
  {...props}
  bind:ref
  bind:value
  class="bg-light-200! caret-primary ring-0 transition-colors hover:bg-light-300! focus:bg-light-200! focus:ring-1 focus:ring-primary {className ??
    ''}"
/>
