<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import TextArea from '#lib/components/TextArea.svelte';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import { descriptionEditor } from '#lib/stores/descriptionEditor.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button } from '@immich/ui';

  const editor = descriptionEditor;
</script>

{#if editor.assets}
  <Dialog
    title="Description"
    subtitle={countOf(editor.assets.length, 'photo')}
    onClose={editor.close}
    bodyClass="flex flex-col gap-4"
  >
    <TextArea
      aria-label="Description"
      bind:value={editor.text}
      placeholder={editor.mixed ? 'Mixed' : 'Add a description'}
      rows={4}
    />
    {#if editor.values.length > 1 || editor.values[0]?.text}
      <section class="flex flex-col gap-1" aria-label="Current descriptions">
        <div class="flex items-center">
          <h3 class="section-label me-auto text-dark/65">Current</h3>
          <Button
            size="tiny"
            variant="ghost"
            color="primary"
            disabled={editor.saving}
            onclick={() => editor.save(true)}
          >
            Clear all
          </Button>
        </div>
        <div class="flex flex-col overflow-hidden rounded-lg ring-1 ring-hairline">
          {#each editor.values as value (value.text)}
            <MenuRow
              label={value.text || 'None'}
              class={value.text ? '' : 'text-dark/65'}
              disabled={!value.text}
              onclick={() => (editor.text = value.text)}
            >
              <span class="shrink-0 text-caption text-dark/65 tabular-nums">{value.count}</span>
            </MenuRow>
          {/each}
        </div>
      </section>
    {/if}
    {#snippet footer()}
      <div class="flex w-full items-center justify-end gap-2">
        <Button
          variant="ghost"
          color="secondary"
          class="touch:hidden"
          disabled={editor.saving}
          onclick={editor.close}
        >
          Cancel
        </Button>
        <Button
          class="touch:h-12 touch:flex-1 touch:rounded-full"
          loading={editor.saving}
          onclick={() => editor.save()}
        >
          Save
        </Button>
      </div>
    {/snippet}
  </Dialog>
{/if}
