<script lang="ts">
  import { page } from '$app/state';
  import PageHeader from '#lib/components/shell/PageHeader.svelte';
  import IconLink from '#lib/components/IconLink.svelte';
  import { Container } from '@immich/ui';
  import { mdiCogOutline, mdiStethoscope } from '@mdi/js';

  let { children }: { children: import('svelte').Snippet } = $props();

  const diagnostics = $derived(page.url.pathname === '/settings/diagnostics');
  const title = $derived(diagnostics ? 'Diagnostics' : 'Settings');
</script>

{#snippet sectionLink()}
  <IconLink
    href={diagnostics ? '/settings' : '/settings/diagnostics'}
    size="medium"
    variant="ghost"
    color="secondary"
    shape="round"
    class="size-11"
    icon={diagnostics ? mdiCogOutline : mdiStethoscope}
    aria-label={diagnostics ? 'Settings' : 'Diagnostics'}
    data-sveltekit-replacestate
  />
{/snippet}

<div class="flex h-full min-h-0 flex-col bg-light-50">
  <PageHeader {title} compactActions={sectionLink} />
  <div class="min-h-0 flex-1 overflow-y-auto">
    <Container center class="max-w-3xl px-4 py-6 sm:px-8">
      <div class="min-w-0">
        {@render children()}
      </div>
    </Container>
  </div>
</div>
