---
layout: default
title: GPU passthrough
parent: Run the server
nav_order: 2
permalink: /gpu-passthrough/
---

# GPU passthrough

A container cannot see the host's graphics card until you hand it over. Passthrough gives the
container two things: the device file the GPU driver talks to, and permission to open it. The image
already contains the Vulkan loader and Mesa's drivers for AMD and Intel. NVIDIA's driver stays on
the host, and NVIDIA Container Toolkit copies it into the container at start.

Passthrough only affects the server: decoding, exports, AI masks and server-drawn previews. Browser
previews use the GPU of the computer you edit from. [Rendering](rendering.md) explains the split.

Without passthrough the server still works. It renders on the CPU through Mesa's software
rasterizer, which is much slower than a real GPU.

## AMD or Intel on Linux

The kernel lists each GPU under `/dev/dri`: a `card` node for displays and a `renderD` node for
rendering. immich-edit only needs the render node.

1. On the host, find the render node and the ID of the group that owns it:

   ```shell
   stat -c '%n group %g (%G)' /dev/dri/renderD*
   ```

   ```text
   /dev/dri/renderD128 group 105 (render)
   ```

   A host with both an integrated and a discrete GPU lists `renderD128` and `renderD129`; the
   server prefers the discrete one. If `/dev/dri` is missing or empty, the host has no GPU driver
   loaded. Enable the driver on the host first, then continue.

1. Pass the directory through and add that group ID:

   ```yaml
   services:
     immich-edit:
       devices:
         - /dev/dri:/dev/dri
       group_add:
         - "105"
   ```

1. Recreate the container:

   ```shell
   docker compose up -d
   ```

Use the number from `stat`, not a group name. The container runs as UID `10001`, and it can only
open the render node as a member of the group that owns it. Docker resolves a group name inside the
container, not on the host: the image has no `render` group, and its `video` group may have a
different ID than the host's. The number means the same group on both sides. NAS systems such as
Unraid often use group IDs that differ from common desktop distributions, so always check.

## NVIDIA on Linux

1. Install the NVIDIA driver on the host, then
   [NVIDIA Container Toolkit](https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/latest/install-guide.html),
   including its step that configures Docker.
1. Reserve the GPU and request its graphics driver:

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

1. Recreate the container and confirm the card is visible:

   ```shell
   docker compose up -d
   docker compose exec immich-edit nvidia-smi
   ```

The reservation decides which GPU the container gets. `NVIDIA_DRIVER_CAPABILITIES` decides which
parts of the driver the toolkit copies in. Its default, `utility,compute`, covers CUDA but not
Vulkan, and immich-edit renders through Vulkan. Without `graphics` the container sees the card
but has no driver for it, and the server quietly uses the software rasterizer. `utility` provides
`nvidia-smi` for the check above.

## Without Compose

`docker run`, and container templates that accept extra `docker run` parameters, take the same
settings as flags.

AMD or Intel, with the group ID from `stat`:

```shell
--device /dev/dri:/dev/dri --group-add 105
```

NVIDIA:

```shell
--gpus all -e NVIDIA_DRIVER_CAPABILITIES=graphics,utility
```

## macOS

Docker cannot pass Metal through its Linux virtual machine, so the container renders with the
software rasterizer. [Run the binary natively](deploy.md#native-execution) to use Metal.

## Check that it worked

Open **Settings** > **Diagnostics** and read the **Server** section:

| Row | With a GPU | Without one |
| --- | --- | --- |
| **Renderer active** | `gpu` | `gpu` or `cpu` |
| **GPU adapter** | The card's name | `llvmpipe` followed by an LLVM version |
| **GPU type** | Not shown | `software rasterizer (no hardware GPU found)` |

**Renderer active** alone proves nothing, because the software rasterizer also counts as a GPU.
Look at **GPU adapter** and **GPU type**.

If the card is missing, look from inside the container:

```shell
docker compose exec immich-edit ls -l /dev/dri
docker compose exec immich-edit id
```

The first command should list the render node, and the second should include its group ID from
the host. `IMMICH_EDIT_RENDERER=cpu` skips the GPU entirely; `gpu` logs why the GPU failed before
falling back. More causes are listed under
[the server does not use the GPU](troubleshooting.md#the-server-does-not-use-the-gpu).
