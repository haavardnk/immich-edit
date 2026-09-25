---
layout: default
title: Backup and upgrade
parent: Run the server
nav_order: 7
permalink: /backup-and-upgrade/
---

# Backup and upgrade

immich-edit keeps everything it owns in one directory, `DATA_DIR`: the database with your edits,
presets and settings, plus masks, imported profiles and models. Your photos are not in it; they
stay in Immich and need Immich's own backup.

In the container, `DATA_DIR` is `/data`. With the Compose file from
[Getting started](getting-started.md), that is the named volume `immich-edit-data`. If you mounted
a host directory instead, such as `./data:/data`, back up that directory.

## Durable data

| Path | Contents |
| --- | --- |
| `immich-edit.db` | Users, sessions, edits, history, copies, presets, jobs, model metadata, and instance configuration |
| `instance.key` | Key used to decrypt stored Immich credentials |
| `dcp/` | Imported and bundled camera profiles referenced by the database |
| `luts/` | Imported `.cube` files referenced by edits |
| `models/` | Downloaded mask models; reinstallable, but part of a complete backup |
| `rasters/` | Brush and generated-mask pixels referenced by saved edits |
| `watermarks/` | Imported export watermark PNGs referenced by the database |

Directories under `cache/` contain regenerable thumbnails, click-model embeddings, export staging
files, and frame-related caches.

{: .warning }
Deleting `rasters/` destroys brush and generated masks. `instance.key` must stay with the database.
Without it, stored credentials cannot be decrypted.

## Back up

For the simplest consistent backup, stop the service and copy the whole directory.

With a named volume:

1. Find the volume's full name. Compose prefixes it with the project, usually the folder name:

   ```shell
   docker volume ls --filter name=immich-edit-data
   ```

   ```text
   DRIVER    VOLUME NAME
   local     immich_immich-edit-data
   ```

1. Stop the service, archive the volume into the current directory, and start it again:

   ```shell
   docker compose stop immich-edit
   docker run --rm -v immich_immich-edit-data:/data -v "$PWD":/backup alpine \
     tar czf /backup/immich-edit-data.tgz -C /data .
   docker compose start immich-edit
   ```

With a host directory, stop the service, copy the directory with `cp -a` or your usual backup tool,
and start it again.

To copy a live database instead, use SQLite's backup command before copying the remaining files:

```shell
sqlite3 /data/immich-edit.db ".backup '/data/immich-edit-backup.db'"
```

## Restore

1. Stop immich-edit.
1. Restore the database, `instance.key`, `dcp/`, `luts/`, `rasters/`, and `watermarks/` from the
   same backup. For
   the archive above:

   ```shell
   docker run --rm -v immich_immich-edit-data:/data -v "$PWD":/backup alpine \
     tar xzf /backup/immich-edit-data.tgz -C /data
   ```

1. Restore `models/` or reinstall models later.
1. Confirm the restored files are writable by container UID and GID `10001`.
1. Start immich-edit and inspect **Settings** > **Diagnostics**.

## Upgrade

1. Back up `DATA_DIR`.
1. Pull and restart:

   ```shell
   docker compose pull
   docker compose up -d
   ```

Database migrations run during startup. If startup fails, keep the backup and read the reported
configuration or migration error. Do not delete `DATA_DIR` to make the error disappear.

## Upgrade to v0.4.0

v0.4.0 changes configuration and the neutral RAW rendering baseline.

### Configuration

- Replace `CACHE_DIR` with `DATA_DIR`.
- Replace every `SEGMENT_*` setting with its `ML_*` equivalent.
- Remove old Immich URL and shared credential environment variables.

The process refuses removed keys instead of ignoring them. Docker users upgrading an old volume
mount the existing volume at `/data` instead of `/cache`.

On the first v0.4.0 start, mask rasters move automatically from `/data/cache/rasters` to
`/data/rasters`. Existing destination files are never overwritten; conflicting old files stay in
`/data/cache/rasters` for review.

### RAW rendering

- Neutral RAW rendering no longer applies a hidden content-dependent exposure gain. Dark captures
  can now appear darker; use **Exposure** or **Auto** when correction is wanted.
- Matched DCP baseline exposure is enabled by default. A sidecar that explicitly disabled it keeps
  that preference.
- **Auto** camera profile falls back to **Default Color**. **Flat** is available for matrix-only
  rendering.
- Capture sharpening is enabled by default for RAW files. Turning it off stores an explicit
  preference.
- A matching lens profile applies automatically to RAW files that have no stored lens preference.
  Existing saved edits migrated from the old boolean field remain explicitly off. New and
  sidecar-free assets use automatic lens correction.

These changes intentionally alter pixels for affected assets. Cached previews and edited
thumbnails regenerate as their render identity changes.

### Older single-user data

The v0.3.0 authentication migration retained older edits under a legacy owner. It did not assign
them to newly provisioned Immich users. There is no reassignment tool. Keep the pre-upgrade backup
if those rows matter.
