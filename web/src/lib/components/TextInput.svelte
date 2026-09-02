<script lang="ts">
  import { Field, IconButton, Input, type InputProps } from '@immich/ui';
  import { mdiEyeOffOutline, mdiEyeOutline } from '@mdi/js';

  type TextInputProps = InputProps & {
    label?: string;
    compact?: boolean;
    color?: 'primary' | 'neutral';
  };

  let {
    label,
    compact = false,
    color,
    ref = $bindable(null),
    containerRef = $bindable(null),
    value = $bindable(''),
    class: className,
    size = compact ? 'tiny' : 'small',
    type = 'text',
    ...props
  }: TextInputProps = $props();

  let passwordVisible = $state(false);

  const inputClass = $derived(
    `[&_input]:caret-primary ${
      color === 'neutral'
        ? 'bg-neutral-800! transition-colors hover:bg-neutral-700! focus-within:bg-neutral-800!'
        : color === 'primary'
          ? 'bg-gray-800! transition-colors hover:bg-gray-700! focus-within:bg-primary/10!'
          : 'focus-within:bg-primary/10!'
    } ${className ?? ''}`
  );
</script>

{#snippet input()}
  {#if type === 'password'}
    <Input
      {...props}
      {size}
      type={passwordVisible ? 'text' : 'password'}
      class={inputClass}
      bind:ref
      bind:containerRef
      bind:value
    >
      {#snippet trailingIcon(disabled)}
        {#if value.length > 0}
          <IconButton
            variant="ghost"
            shape="round"
            color="secondary"
            {size}
            class="me-1"
            icon={passwordVisible ? mdiEyeOffOutline : mdiEyeOutline}
            onclick={() => (passwordVisible = !passwordVisible)}
            {disabled}
            title={passwordVisible ? 'Hide password' : 'Show password'}
            aria-label={passwordVisible ? 'Hide password' : 'Show password'}
          />
        {/if}
      {/snippet}
    </Input>
  {:else}
    <Input {...props} {size} {type} class={inputClass} bind:ref bind:containerRef bind:value />
  {/if}
{/snippet}

{#if label}
  <Field
    {label}
    {size}
    class={compact ? 'editor-compact-input-field' : undefined}
    disabled={props.disabled === true}
  >
    {@render input()}
  </Field>
{:else}
  {@render input()}
{/if}
