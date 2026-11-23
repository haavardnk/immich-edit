---
title: Install
description: Deploy immich-edit with Docker Compose and connect it to Immich.
---

## Requirements

- Running Immich server reachable from the container
- Docker Engine with `docker compose`
- 8 GB RAM recommended for RAW rendering and optional AI models
- Browser on desktop, tablet, or phone (WebGPU enables local previews over HTTPS or `localhost`)

## Start the container

Create `compose.yaml`:

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

Start it and open `http://<server-address>:3000`:

```shell
docker compose up -d
```

:::caution
Complete setup before exposing the port publicly. The first Immich administrator to finish setup
claims the instance.
:::

## Claim the instance

1. Enter your Immich server URL (a Docker network address such as `http://immich-server:2283` works)
   and select **Continue**.
2. Sign in with an Immich administrator account via OAuth, email and password, or an Immich API
   key, then select **Connect and claim instance**.

OAuth appears automatically when enabled in Immich, once immich-edit's URLs are registered with
your identity provider. See [OAuth sign-in](../hosting/oauth.md).

## Next steps

- Put immich-edit behind HTTPS for secure cookies, browser WebGPU previews, and app installation:
  [Deployment](../hosting/deployment.md).
- Pass through a Linux GPU for fast server renders and exports:
  [GPU passthrough](../hosting/gpu.md).
- Ready to edit? Open the [Workspace guide](../editing/workspace.md).
