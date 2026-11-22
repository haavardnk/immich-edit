---
title: Backup and upgrade
description: Back up durable DATA_DIR files, restore safely, and follow version upgrade notes.
---

All durable state lives in `DATA_DIR` (`/data` in Docker). Originals stay in Immich and are backed
up separately with Immich.

## Durable files

| Path | Contents |
| --- | --- |
| `immich-edit.db` | Users, sessions, edits, history, copies, presets, jobs, settings |
| `instance.key` | Encryption key for stored Immich credentials |
| `dcp/`, `luts/`, `watermarks/` | Imported camera profiles, `.cube` LUTs, export watermarks |
| `rasters/` | Brush and generated mask pixels referenced by saved edits |
| `models/` | Downloaded AI models (reinstallable) |

Everything under `cache/` is regenerable.

:::caution
Never delete `rasters/` (destroys masks) and always keep `instance.key` paired with
`immich-edit.db`.
:::

## Back up and restore

Stop the container, archive `DATA_DIR`, and restart:

```shell
docker volume ls --filter name=immich-edit-data
docker compose stop immich-edit
docker run --rm -v immich_immich-edit-data:/data -v "$PWD":/backup alpine \
  tar czf /backup/immich-edit-data.tgz -C /data .
docker compose start immich-edit
```

For a live SQLite snapshot instead:

```shell
sqlite3 /data/immich-edit.db ".backup '/data/immich-edit-backup.db'"
```

To restore: stop the container, extract the archive into `/data` (ensure ownership/write access for
UID/GID `10001`), start, and check **Settings** > **Diagnostics**.

## Upgrade

Back up `DATA_DIR`, then pull and recreate:

```shell
docker compose pull && docker compose up -d
```

Database migrations run automatically on startup. Never delete `DATA_DIR` on a migration error —
read the log and restore your backup if rolling back.

## Notable version notes

- **v0.8.0**: schema adds account preferences, saved views, snapshots, and export outputs (not
  downgradeable to v0.7.2 without backup restore). **Geometry** renamed to **Crop** (`Shift+R`);
  browser preview setting moved to **Settings** > **Preferences** > **Preview renderer**.
- **v0.7.2**: adds per-user appearance table (not downgradeable to v0.7.1).
- **v0.7.0**: migrates RAW white balance to Kelvin; Immich API keys need `asset.delete` for trash
  and `asset.copy` for **Replace original**.
- **v0.4.0**: removed `CACHE_DIR`, `SEGMENT_*`, and old Immich env vars; see
  [Removed settings](configuration.md#removed-settings).
