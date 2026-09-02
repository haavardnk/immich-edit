<script lang="ts">
  import { live } from '$lib/api/health';
  import Wordmark from '$lib/components/Wordmark.svelte';
  import { Button, Card, CardBody, Container, Heading, Text } from '@immich/ui';

  let { retry }: { retry: () => void } = $props();

  const DELAYS = [1000, 2000, 4000, 8000, 10000];

  let attempt = $state(0);
  let probing = $state(false);

  async function probe(): Promise<boolean> {
    probing = true;
    try {
      await live();
      return true;
    } catch {
      return false;
    } finally {
      probing = false;
    }
  }

  async function attemptNow(): Promise<void> {
    if (probing) return;
    if (await probe()) {
      attempt = 0;
      retry();
    } else {
      attempt += 1;
    }
  }

  $effect(() => {
    const delay = DELAYS[Math.min(attempt, DELAYS.length - 1)];
    const timer = setTimeout(() => void attemptNow(), delay);
    return () => clearTimeout(timer);
  });
</script>

<div class="flex h-full w-full items-center p-6">
  <Container size="small" center>
    <Card
      color="secondary"
      shape="rectangle"
      class="mx-auto max-w-md rounded-lg border border-dark/10"
    >
      <CardBody class="p-6">
        <div class="flex flex-col gap-4">
          <Heading tag="h1" size="small">
            <Wordmark size="small" />
          </Heading>
          <Heading tag="h2" size="tiny">Can't reach the immich-edit server.</Heading>
          <Text size="small" color="muted">
            It may still be starting up, or it may not be running. This page reconnects
            automatically.
          </Text>
          <Button
            type="button"
            size="small"
            color="primary"
            loading={probing}
            disabled={probing}
            onclick={() => void attemptNow()}
          >
            Retry now
          </Button>
        </div>
      </CardBody>
    </Card>
  </Container>
</div>
