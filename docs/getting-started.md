---
layout: default
title: Getting started
nav_order: 3
permalink: /getting-started/
---

# Getting started

Use Docker Compose for a first installation.

## Requirements

- A running Immich server reachable from the immich-edit container
- Docker Engine with the `docker compose` plugin
- A desktop browser for the editor
- 8 GB RAM recommended for RAW rendering and local mask models

## Start the container

1. Create `compose.yaml`:

   ```yaml
   services:
     immich-edit:
       image: haavardnk/immich-edit:latest
       ports:
         - "3000:3000"
       volumes:
         - immich-edit-data:/data
       restart: unless-stopped

   volumes:
     immich-edit-data:
   ```

1. Start the service:

   ```shell
   docker compose up -d
   ```

1. Open `http://<server-address>:3000`.

{: .warning }
Complete setup before exposing the service publicly. The first Immich administrator to finish the
setup form claims an unconfigured instance.

## Claim the instance

1. Enter the Immich server URL.
1. Choose password or API key authentication.
1. Sign in with an Immich administrator account.

Other users sign in with their own Immich account; each sees only their own edits, presets, and
jobs. OAuth-only Immich accounts use an API key.

## Next steps

- [Back up `DATA_DIR`](backup-and-upgrade.md)
- [Enable GPU rendering](deploy.md#gpu-passthrough)
- [Install optional mask models](administration.md#mask-models)
- [Learn the keyboard shortcuts](shortcuts.md)
