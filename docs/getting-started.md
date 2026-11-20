---
layout: default
title: Getting started
nav_order: 2
permalink: /getting-started/
---

# Getting started

Use Docker Compose for a first installation.

## Requirements

- A running Immich server reachable from the immich-edit container
- Docker Engine with the `docker compose` plugin
- A browser on a computer, tablet or phone. With WebGPU the editor can draw previews itself; on a
  plain `http://` LAN address that takes
  [one extra step](rendering.md#browser-previews-on-a-local-network)
- 8 GB RAM recommended for RAW rendering and local AI models

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

1. Enter the Immich server URL and select **Continue**. A Docker network address such as
   `http://immich-server:2283` works too.
1. Sign in with an Immich administrator account, using the OAuth button, email and password, or
   an API key, and select **Connect and claim instance**.

The OAuth button appears when Immich has OAuth enabled. It only works once immich-edit's addresses
are registered with your identity provider; see [OAuth sign-in](oauth.md).

Other users sign in with their own Immich account; each sees only their own edits, presets, and
jobs.

## Phones and tablets

The same address works on a phone or tablet, with a touch layout; see
[on a phone or tablet](cull.md#on-a-phone-or-tablet). **Install** beside **Install app** in
**Settings** > **Preferences** adds immich-edit to the Home Screen or installs it on a computer. It
needs an [HTTPS address](deploy.md#https-for-the-installed-app) with a trusted certificate.

## Next steps

- [Put it behind HTTPS](deploy.md#reverse-proxy), or pick another way to get
  [browser previews on a local network](rendering.md#browser-previews-on-a-local-network).
- [Pass a GPU through](gpu-passthrough.md) for faster exports and AI masks.
- [Install models](administration.md#models) if you want AI masks or **AI Denoise**.
- [Start editing](use.md).
