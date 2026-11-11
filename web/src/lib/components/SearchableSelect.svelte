<script lang="ts" generics="T">
  import TextInput from '#lib/components/TextInput.svelte';
  import { Button, Icon, IconButton } from '@immich/ui';
  import { mdiCheck, mdiChevronDown, mdiClose, mdiEyeOutline, mdiMagnify, mdiPlus } from '@mdi/js';
  import { Combobox, Command } from 'bits-ui';
  import BottomSheet from '#lib/components/sheet/BottomSheet.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { pickerHost } from '#lib/components/sheet/pickerHost.ts';
  import { presence } from '#lib/components/sheet/presence.svelte.ts';
  import { flushSync, untrack, type Snippet } from 'svelte';
  import { rowHold } from '#lib/gestures/rowHold.svelte.ts';

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
    compact = false,
    multiple = true,
    color = 'primary',
    disabled = false,
    side = 'bottom',
    autofocus = false,
    closeOnSelect = true,
    commitOnClose = false,
    hideSelected = multiple,
    previewing = null,
    onSelectedChange,
    onCreate,
    onPreview
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
    compact?: boolean;
    multiple?: boolean;
    color?: 'primary' | 'neutral';
    disabled?: boolean;
    side?: 'top' | 'bottom';
    autofocus?: boolean;
    closeOnSelect?: boolean;
    commitOnClose?: boolean;
    hideSelected?: boolean;
    previewing?: string | null;
    onSelectedChange?: (selected: string[]) => void;
    onCreate?: (value: string) => void | Promise<void>;
    onPreview?: (item: T | null) => void;
  } = $props();

  const OPTION_CLASS =
    'flex min-h-8 w-full shrink-0 cursor-pointer items-center gap-2 px-3 py-1 text-xs outline-none select-none touch:min-h-11 touch:gap-3 touch:px-4 touch:text-sm';
  const PREVIEW_KEYS = new Set(['ArrowDown', 'ArrowUp', 'Home', 'End', 'PageDown', 'PageUp']);

  const host = pickerHost();
  const hold = rowHold({
    attribute: 'data-option-id',
    start: (id) => {
      const item = options.find((option) => getId(option) === id);
      if (item) preview(item);
    },
    end: endPreview
  });

  let previewed = false;
  let previewKeys = false;

  let input = $state('');
  let open = $state(false);
  const sheet = presence(() => open);
  let anchor = $state<HTMLElement | null>(null);
  let inputElement = $state<HTMLInputElement | null>(null);
  let sheetInput = $state<HTMLInputElement | null>(null);
  const selectedSet = $derived(new Set(selected));
  const selectedItem = $derived(
    multiple ? null : (options.find((item) => selectedSet.has(getId(item))) ?? null)
  );
  const selectedLabel = $derived(selectedItem ? getLabel(selectedItem) : '');
  const inputText = $derived(open ? input : selectedLabel);
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
  const comboboxHighlight = $derived(
    color === 'neutral'
      ? 'hover:bg-light-950/6 data-highlighted:bg-light-950/6'
      : 'hover:bg-primary-200 data-highlighted:bg-primary-200'
  );
  const commandHighlight = $derived(
    color === 'neutral' ? 'data-selected:bg-light-950/6' : 'data-selected:bg-primary-200'
  );

  function setOpen(next: boolean): void {
    if (next && !open) input = '';
    open = next;
  }

  function preview(item: T): void {
    if (!onPreview) return;
    previewed = true;
    onPreview(item);
  }

  function endPreview(): void {
    previewKeys = false;
    if (!previewed) return;
    previewed = false;
    onPreview?.(null);
  }

  function hover(event: PointerEvent, item: T): void {
    if (event.pointerType !== 'touch') preview(item);
  }

  function unhover(event: PointerEvent): void {
    if (event.pointerType !== 'touch') endPreview();
  }

  function keydown(event: KeyboardEvent): void {
    previewKeys = PREVIEW_KEYS.has(event.key);
    createOnEnter(event);
  }

  function typed(value: string): void {
    input = value;
    endPreview();
  }

  $effect(() => {
    if (!open) untrack(endPreview);
  });

  $effect(() => () => untrack(endPreview));

  function change(value: string[]): void {
    const next = multiple ? value : value.slice(-1);
    selected = next;
    input = '';
    setOpen(!closeOnSelect);
    onSelectedChange?.(next);
  }

  function toggle(id: string): void {
    change(selectedSet.has(id) ? selected.filter((value) => value !== id) : [...selected, id]);
  }

  function focusInput(): void {
    setOpen(true);
    queueMicrotask(() => inputElement?.focus());
  }

  function openSheet(): void {
    setOpen(true);
    flushSync();
    sheetInput?.focus({ preventScroll: true });
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
    setOpen(!closeOnSelect);
  }

  function createOnEnter(event: KeyboardEvent): void {
    if (event.key !== 'Enter' || !canCreate || suggestions.length > 0) return;
    event.preventDefault();
    void create();
  }

  function commitTyped(next: boolean): void {
    if (next || !commitOnClose || !createValue) return;
    const match = options.find(
      (option) => getLabel(option).toLowerCase() === createValue.toLowerCase()
    );
    if (match) change([...selected, getId(match)]);
    else void create();
  }

  function sheetOpenChange(next: boolean): void {
    setOpen(next);
    commitTyped(next);
  }
</script>

{#snippet optionBody(item: T)}
  {@const description = getDescription?.(item)}
  {@const itemColor = getColor?.(item)}
  {#if itemColor}
    <span class="size-2 shrink-0 rounded-full" style:background-color={itemColor}></span>
  {/if}
  <span class="min-w-0 flex-1 text-left">
    <span class="block truncate" title={getLabel(item)}>{getLabel(item)}</span>
    {#if description}
      <span class="block truncate text-[10px] text-dark/65">{description}</span>
    {/if}
  </span>
  {#if previewing !== null && previewing === getId(item)}
    <span class="flex shrink-0 items-center gap-1 text-[10px] text-primary touch:text-xs">
      <Icon icon={mdiEyeOutline} size="12px" aria-hidden="true" />
      previewing
    </span>
  {/if}
  {#if selectedSet.has(getId(item))}
    <Icon icon={mdiCheck} size="14px" class="shrink-0 text-primary" />
  {/if}
{/snippet}

{#snippet comboboxRow(item: T)}
  <Combobox.Item
    value={getId(item)}
    label={getLabel(item)}
    class="{OPTION_CLASS} {comboboxHighlight}"
    onHighlight={() => {
      if (previewKeys) preview(item);
    }}
    onpointerenter={(event: PointerEvent) => hover(event, item)}
  >
    {@render optionBody(item)}
  </Combobox.Item>
{/snippet}

{#snippet commandRow(item: T)}
  <Command.Item
    value={getId(item)}
    class="{OPTION_CLASS} {commandHighlight}"
    data-option-id={onPreview ? getId(item) : undefined}
    onSelect={() => toggle(getId(item))}
  >
    {@render optionBody(item)}
  </Command.Item>
{/snippet}

{#snippet optionList(row: Snippet<[T]>)}
  {#each suggestions as item, index (getId(item))}
    {@const group = getGroup?.(item)}
    {@const previous = suggestions[index - 1]}
    {@const previousGroup = previous ? getGroup?.(previous) : null}
    {#if group && group !== previousGroup}
      <div class="px-3 pt-2 pb-1 text-[9px] font-semibold text-dark/65 uppercase">
        {group}
      </div>
    {/if}
    {@render row(item)}
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
      class="h-9 w-full justify-start rounded-none px-3 text-left text-xs touch:h-11 touch:px-4 touch:text-sm {color ===
      'neutral'
        ? 'hover:bg-light-950/6'
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
{/snippet}

{#snippet searchIcon()}
  <Icon icon={mdiMagnify} size="60%" aria-hidden="true" />
{/snippet}

{#snippet chevronIcon()}
  <Icon icon={mdiChevronDown} size="60%" aria-hidden="true" />
{/snippet}

{#snippet picker()}
  <Command.Root shouldFilter={false} loop disablePointerSelection class="flex flex-col text-dark">
    <div class="sticky top-0 z-10 bg-light-50 px-4 pt-3 pb-2">
      <Command.Input
        value={input}
        placeholder={searchPlaceholder}
        aria-label={label}
        {disabled}
        oninput={(event) => (input = event.currentTarget.value)}
        onkeydown={createOnEnter}
      >
        {#snippet child({ props })}
          <TextInput
            {color}
            bind:ref={sheetInput}
            size="small"
            leadingIcon={searchIcon}
            {...props}
            value={input}
          />
        {/snippet}
      </Command.Input>
    </div>
    <div class={onPreview ? 'no-callout' : ''} {@attach onPreview && hold.attachment}>
      <Command.List aria-label={label} class="pb-2 outline-none">
        {@render optionList(commandRow)}
      </Command.List>
    </div>
  </Command.Root>
{/snippet}

{#if layout.mode === 'touch' && host.inline}
  {@render picker()}
{:else if layout.mode === 'touch'}
  <div class="relative contain-inline-size">
    <Button
      size={compact ? 'tiny' : 'small'}
      variant="outline"
      color="secondary"
      fullWidth
      {disabled}
      aria-label={label}
      aria-haspopup="dialog"
      aria-expanded={open}
      class="min-h-11 justify-between font-normal {selectedLabel ? 'pr-12' : ''}"
      onclick={openSheet}
    >
      <span class="min-w-0 truncate {selectedLabel ? '' : 'text-dark/65'}">
        {selectedLabel || placeholder}
      </span>
      {#if !selectedLabel}
        <Icon icon={mdiChevronDown} size="16px" class="shrink-0" aria-hidden="true" />
      {/if}
    </Button>
    {#if selectedLabel}
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        class="absolute top-1/2 right-0 min-h-11 min-w-11 -translate-y-1/2"
        icon={mdiClose}
        aria-label="Clear {selectedLabel}"
        onclick={clear}
      />
    {/if}
  </div>
  {#if sheet.mounted}
    <BottomSheet
      {open}
      onOpenChange={sheetOpenChange}
      onExited={sheet.exited}
      title={label}
      close="icon"
      detents={[{ fraction: 1 }]}
      peek={hold.held !== null}
      data-searchable-select-sheet
    >
      {@render picker()}
    </BottomSheet>
  {/if}
{:else}
  <Combobox.Root
    type="multiple"
    {disabled}
    bind:value={selected}
    bind:open={() => open, setOpen}
    inputValue={inputText}
    onValueChange={change}
    onOpenChange={commitTyped}
  >
    <div class="relative contain-inline-size">
      <Combobox.Input
        placeholder={open ? searchPlaceholder : placeholder}
        aria-label={label}
        onfocus={focusInput}
        onclick={focusInput}
        oninput={(event) => typed(event.currentTarget.value)}
        onkeydown={keydown}
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
            value={inputText}
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
        customAnchor={anchor}
        onpointerleave={unhover}
        class="editor-compact-combobox-content z-50 w-(--bits-floating-anchor-width) rounded-b-lg border border-light-950/10 py-1 text-dark data-[side=top]:rounded-t-lg data-[side=top]:rounded-b-none {color ===
        'neutral'
          ? 'bg-light-200'
          : 'bg-gray-100 dark:bg-gray-800'}"
      >
        <div
          class="overflow-y-auto {compact
            ? 'max-h-(--bits-combobox-content-available-height)'
            : 'max-h-62'}"
        >
          {@render optionList(comboboxRow)}
        </div>
      </Combobox.Content>
    </Combobox.Portal>
  </Combobox.Root>
{/if}
