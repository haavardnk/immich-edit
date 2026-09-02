<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { Button, Card, CardBody, Container, Heading, Text } from '@immich/ui';

  function reload(): void {
    location.reload();
  }

  function home(): void {
    void goto('/photos');
  }
</script>

<div class="flex min-h-screen items-center bg-light p-6 text-dark">
  <Container size="small" center>
    <Card
      color="secondary"
      shape="rectangle"
      class="mx-auto max-w-md rounded-lg border border-dark/10"
    >
      <CardBody class="p-6">
        <div class="flex flex-col gap-4">
          <Heading tag="h1" size="small">Something went wrong</Heading>
          <Text size="small" color="muted">{page.error?.message ?? 'Unexpected error'}</Text>
          {#if page.status}
            <Text size="tiny" color="muted">Status {page.status}</Text>
          {/if}
          <div class="flex gap-2">
            <Button size="tiny" color="primary" onclick={reload}>Reload</Button>
            <Button size="tiny" variant="ghost" color="secondary" onclick={home}>
              Go to Photos
            </Button>
          </div>
        </div>
      </CardBody>
    </Card>
  </Container>
</div>
