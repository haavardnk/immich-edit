# immich-edit web

SvelteKit frontend for immich-edit. It builds as a static SPA and talks to the Rust backend through `/api/*`.

## Setup

```bash
npm install
```

## Development

```bash
npm run dev
```

The dev server proxies API requests to the backend. Run the backend from the repo root in another terminal.

## Build

```bash
npm run build
```

The Docker image uses this build output. For local full-app development, use the root [CONTRIBUTING.md](../CONTRIBUTING.md).
