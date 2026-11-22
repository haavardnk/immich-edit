---
title: GPU passthrough
description: Pass AMD, Intel, or NVIDIA GPUs into the immich-edit Docker container.
---

The image ships the Vulkan loader and Mesa drivers for AMD and Intel. NVIDIA drivers are injected
from the host by NVIDIA Container Toolkit. Without a GPU, the container falls back to Mesa's
software rasterizer (`llvmpipe`).

Passthrough accelerates server work: RAW decode, source preparation, exports, edited thumbnails,
and AI models. Browser previews use the client device's GPU (see [Rendering](rendering.md)).

## AMD or Intel (Linux)

1. Find the render node and its numeric group ID on the host:

   ```shell
   stat -c '%n group %g (%G)' /dev/dri/renderD*
   ```

2. Add the device and numeric GID to `compose.yaml` (always use the number — the container runs as
   UID `10001` and resolves group names inside the container, not the host):

   ```yaml
   services:
     immich-edit:
       devices:
         - /dev/dri:/dev/dri
       group_add:
         - "105"
   ```

3. Recreate the container:

   ```shell
   docker compose up -d
   ```

## NVIDIA (Linux)

1. Install the host NVIDIA driver and NVIDIA Container Toolkit configured for Docker.
2. Add the GPU reservation and `graphics` capability (the default `utility,compute` omits Vulkan):

   ```yaml
   services:
     immich-edit:
       environment:
         NVIDIA_DRIVER_CAPABILITIES: graphics,utility
       deploy:
         resources:
           reservations:
             devices:
               - driver: nvidia
                 count: 1
                 capabilities: [gpu]
   ```

3. Recreate and verify:

   ```shell
   docker compose up -d
   docker compose exec immich-edit nvidia-smi
   ```

## Without Compose

```shell
--device /dev/dri:/dev/dri --group-add 105
```

```shell
--gpus all -e NVIDIA_DRIVER_CAPABILITIES=graphics,utility
```

## macOS

Docker on macOS cannot pass Metal through its Linux VM. Run natively to use Metal (see
[Deployment](deployment.md#native-execution)).

## Verify

Open **Settings** > **Diagnostics** > **Server**:

| Row | Hardware GPU | Software fallback |
| --- | --- | --- |
| **Renderer active** | `gpu` | `gpu` or `cpu` |
| **GPU adapter** | Your card name | `llvmpipe (...)` |
| **GPU type** | Not shown | `software rasterizer (no hardware GPU found)` |

If the card is missing, inspect permissions inside the container:

```shell
docker compose exec immich-edit ls -l /dev/dri
docker compose exec immich-edit id
```
