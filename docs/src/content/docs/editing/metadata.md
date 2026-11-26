---
title: Metadata
description: Fix capture times and time zones for owned photos in Immich.
---

Select photos in the grid and open **Metadata** in the bulk bar (**More** > **Metadata** on touch).
It holds **Capture time** and **Location**; changes write back to Immich for photos you own.

## Capture time

**Capture time** previews the selection in its new order, flagging photos that move with
**Reordered**. Select **Apply** to write to Immich, or **Undo** in the toast to restore every time.

- **Shift**: move capture times by days, hours, minutes, and seconds (type `+1:02:37` or
  `-3 d 4:00:00`, or use the steppers).
- **Set by a photo**: pick a photo in the preview and enter the exact time it should show (such as
  a photo of a clock), or match it to a photo from another camera taken at the same moment. The
  difference becomes that camera's shift.
- **One time for all**: assign one date and time to the selection, spaced one second apart by file
  name so scans keep their order.

When a selection includes multiple cameras, **Camera** scopes shifts and time zones per camera so a
drifted camera can be aligned with a phone (**One time for all** always updates every photo).

**Time zone** starts at **As shot** and switches the photos to another zone. Once changed, **Camera
time zone** sets the zone the camera was left on (for example `13:05 on the camera → 19:05 in
Bangkok`). Unzoned photos inherit your device's zone when their time changes.

For a single photo, open **Photo info** (`I`) and select its **Taken** value.
