<script lang="ts" generics="T">
  import TextInput from '$lib/components/TextInput.svelte';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiCheck, mdiChevronDown, mdiClose, mdiMagnify, mdiPlus } from '@mdi/js';
  import { Combobox } from 'bits-ui';

  let {
    options,
    selected = $bindable(),
    getId,
    getLabel,
    getColor,
    getDescription,
    getGroup,
    getSearchText = getLabel,
    placeholder = 'Search…',
    label = placeholder,
    summary = '',
    compact = false,
    multiple = true,
    color = 'primary',
    disabled = false,
    side = 'bottom',
    autofocus = false,
    closeOnSelect = true,
    hideSelected = multiple,
    onSelectedChange,
    onCreate
  }: {
    options: T[];
    selected: string[];
    getId: (item: T) => string;
    getLabel: (item: T) => string;
    getColor?: (item: T) => string | null | undefined;
    getDescription?: (item: T) => string | null | undefined;
    getGroup?: (item: T) => string | null | undefined;
    getSearchText?: (item: T) => string;
    placeholder?: string;
    label?: string;
    summary?: string;
    compact?: boolean;
    multiple?: boolean;
    color?: 'primary' | 'neutral';
    disabled?: boolean;
    side?: 'top' | 'bottom';
    autofocus?: boolean;
    closeOnSelect?: boolean;
    hideSelected?: boolean;
    onSelectedChange?: (selected: string[]) => void;
    onCreate?: (value: string) => void | Promise<void>;
  } = $props();

  let input = $state('');
  let open = $state(false);
  let anchor = $state<HTMLElement | null>(null);
  let inputElement = $state<HTMLInputElement | null>(null);

  const selectedSet = $derived(new Set(selected));
  const selectedItem = $derived(
    multiple ? null : (options.find((item) => selectedSet.has(getId(item))) ?? null)
  );
  const selectedLabel = $derived(
    multiple ? (selected.length > 0 ? summary : '') : selectedItem ? getLabel(selectedItem) : ''
  );
  const searchPlaceholder = $derived(placeholder.replace(/^(Add|Choose|Select) /, 'Search '));
  const createValue = $derived(input.trim());
  const suggestions = $derived.by(() => {
    const query = input.trim().toLowerCase();
    return options
      .filter((item) => !hideSelected || !selectedSet.has(getId(item)))
      .filter((item) => !query || getSearchText(item).toLowerCase().includes(query));
  });
  const canCreate = $derived(
    onCreate != null &&
      createValue.length > 0 &&
      !options.some((option) => getLabel(option).toLowerCase() === createValue.toLowerCase())
  );

  function change(value: string[]): void {
    const next = multiple ? value : value.slice(-1);
    selected = next;
    input = '';
    open = !closeOnSelect;
    onSelectedChange?.(next);
  }

  function focusInput(): void {
    open = true;
    queueMicrotask(() => inputElement?.focus());
  }

  function clear(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    selected = [];
    input = '';
    onSelectedChange?.([]);
  }

  async function create(): Promise<void> {
    if (!onCreate || !canCreate) return;
    await onCreate(createValue);
    input = '';
    open = !closeOnSelect;
  }
</script>

{#snippet searchIcon()}
  <Icon icon={mdiMagnify} size="60%" aria-hidden="true" />
{/snippet}

{#snippet chevronIcon()}
  <Icon icon={mdiChevronDown} size="60%" aria-hidden="true" />
{/snippet}

<Combobox.Root
  type="multiple"
  {disabled}
  bind:value={selected}
  bind:open
  inputValue={open ? input : selectedLabel}
  onValueChange={change}
>
  <div class="relative contain-inline-size">
    <Combobox.Input
      placeholder={open ? searchPlaceholder : placeholder}
      aria-label={label}
      onfocus={focusInput}
      onclick={focusInput}
      oninput={(event) => (input = event.currentTarget.value)}
    >
      {#snippet child({ props })}
        <TextInput
          {autofocus}
          {color}
          bind:ref={inputElement}
          size={compact ? 'tiny' : 'small'}
          class="editor-compact-combobox {compact ? '' : 'ring-0'} {open
            ? 'editor-compact-combobox-open ring-1 ring-primary'
            : 'editor-compact-combobox-closed ring-0'} {selectedLabel ? '[&_input]:pr-8' : ''}"
          leadingIcon={open || selectedLabel ? searchIcon : undefined}
          trailingIcon={!open && !selectedLabel ? chevronIcon : undefined}
          bind:containerRef={anchor}
          {...props}
        />
      {/snippet}
    </Combobox.Input>
    {#if selectedLabel && !open}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        class="absolute top-1/2 right-0.5 -translate-y-1/2"
        icon={mdiClose}
        title="Clear {selectedLabel}"
        aria-label="Clear {selectedLabel}"
        onmousedown={(event: MouseEvent) => event.preventDefault()}
        onclick={clear}
      />
    {/if}
  </div>

  <Combobox.Portal>
    <Combobox.Content
      {side}
      sideOffset={-1}
      collisionPadding={8}
      avoidCollisions={!compact}
      customAnchor={anchor}
      class="editor-compact-combobox-content z-50 w-(--bits-floating-anchor-width) rounded-b-lg border border-white/10 py-1 text-dark data-[side=top]:rounded-t-lg data-[side=top]:rounded-b-none {color ===
      'neutral'
        ? 'bg-neutral-800'
        : 'bg-gray-800'}"
    >
      <div
        class="overflow-y-auto {compact
          ? 'max-h-(--bits-combobox-content-available-height)'
          : 'max-h-62'}"
      >
        {#each suggestions as item, index (getId(item))}
          {@const group = getGroup?.(item)}
          {@const previous = suggestions[index - 1]}
          {@const previousGroup = previous ? getGroup?.(previous) : null}
          {@const description = getDescription?.(item)}
          {@const itemColor = getColor?.(item)}
          {#if group && group !== previousGroup}
            <div class="px-3 pt-2 pb-1 text-[9px] font-semibold text-dark/65 uppercase">
              {group}
            </div>
          {/if}
          <Combobox.Item
            value={getId(item)}
            label=""
            class="flex min-h-8 w-full shrink-0 cursor-pointer items-center gap-2 px-3 py-1 text-xs outline-none select-none {color ===
            'neutral'
              ? 'hover:bg-white/6 data-highlighted:bg-white/6'
              : 'hover:bg-primary-200 data-highlighted:bg-primary-200'}"
          >
            {#if itemColor}
              <span class="size-2 shrink-0 rounded-full" style:background-color={itemColor}></span>
            {/if}
            <span class="min-w-0 flex-1 text-left">
              <span class="block truncate" title={getLabel(item)}>{getLabel(item)}</span>
              {#if description}
                <span class="block truncate text-[10px] text-dark/65">{description}</span>
              {/if}
            </span>
            {#if selectedSet.has(getId(item))}
              <Icon icon={mdiCheck} size="14px" class="shrink-0 text-primary" />
            {/if}
          </Combobox.Item>
        {:else}
          {#if !canCreate}
            <p class="flex h-9 items-center px-3 text-xs text-muted">No matches</p>
          {/if}
        {/each}
        {#if canCreate}
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            class="h-9 w-full justify-start rounded-none px-3 text-left text-xs {color === 'neutral'
              ? 'hover:bg-white/6'
              : 'hover:bg-primary-200'}"
            onmousedown={(event: MouseEvent) => {
              event.preventDefault();
              void create();
            }}
          >
            <Icon icon={mdiPlus} size="14px" />
            <span class="min-w-0 truncate">Create "{createValue}"</span>
          </Button>
        {/if}
      </div>
    </Combobox.Content>
  </Combobox.Portal>
</Combobox.Root>
