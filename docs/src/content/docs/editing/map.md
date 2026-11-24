---
title: Map and geotagging
description: Browse geotagged photos on a map and set or move photo locations.
---

Open **Map** from the sidebar (under **Library** > **Collections** on phones) to explore your
geotagged library and assign missing locations.

## Browse the map

- Clusters expand into individual photo markers as you zoom in.
- Filter the map by **Favorites** and date range, or select **Fit all** to frame every matching
  marker.
- On desktop and landscape tablets, the side panel lists photos in the current map view; on phones
  and portrait tablets, drag the bottom sheet up to browse them.
- Open any photo in the loupe or editor and return directly to your map position.

<!-- SCREENSHOT: capture.cjs map-desktop + compose.cjs map-desktop -->
![Desktop map view with date filter, photo clusters, and photos-in-view panel](../../../assets/shots/map-desktop.png)

<!-- SCREENSHOT: capture.cjs map-phone + compose.cjs map-phone -->
![Phone map view with clusters and expandable bottom sheet](../../../assets/shots/map-phone.png)

## Set or move locations

Locations can be set or updated on photos you own (Immich does not support clearing GPS coordinates
in bulk, so a location can be moved but not removed):

- **Single photo**: open **Photo info** (`I`) to see the mini map, then choose **Set location** or
  **Edit location**. Search by place name or drag the pin, then select **Save**.
- **Coordinates and map links**: paste coordinates (`60.3913, 5.3221` or `60°23'29"N 5°19'20"E`)
  or a Google Maps, Apple Maps, OpenStreetMap or `geo:` link into the search field and choose
  **Go to**. They are read in your browser, so this works even when place search is turned off.
- **Multiple photos**: select photos in the grid and open **Location** in the bulk bar (or the
  **More** sheet on touch), then **Choose on map**.
- **Drag onto the map**: drag tiles from the map panel — including the **No location** tab — onto
  the map to geotag them, or drag an existing marker to refine its position (press and hold first
  on touch).
- **Copy and paste**: copy a photo's location from **Photo info**, the **Location** menu or with
  `Alt+Shift+C`, then paste it onto any selection with **Paste location** or `Alt+Shift+V`. The
  copied place lasts until you close the tab, and its coordinates also land on the system clipboard
  as text for other apps.

## Saved and recent places

- **Save a place**: in the location picker, select the star beside the place name, give it a
  name and choose **Save place**. The star stays filled whenever the pin sits on a saved place.
- **Use a place**: focus the search field to see your saved places and the last places you set,
  or type part of a name. The **Location** menu in the bulk bar lists them too, so a selection
  can be placed in one click.
- **On the map**: saved places show as stars (toggle them with the star in the map header).
  Photos dropped within a finger's width of a star land exactly on it, and selecting a star copies
  its location or places the selected photos there.

<!-- SCREENSHOT: capture.cjs location-desktop + compose.cjs location-desktop -->
![Set location dialog with Photon place search and draggable map pin](../../../assets/shots/location-desktop.png)

## Match photos to a track

Place a whole shoot from a GPX, KML or TCX track recorded by a watch, phone app or GPS logger.

1. Select the photos and choose **Location** > **Match to a track**, or select **Match track** in
   the map header to work on every photo taken during the track.
2. Drop one or more track files on the map, or choose them. Files are read in your browser and never
   uploaded; only the locations you place are saved to Immich.
3. Line up the camera clock. **Best fit** proposes the correction that puts the most photos on the
   track, and you can adjust it by the minute or hour or type it (`-1:02:14`). Photos that recorded a
   time zone are matched exactly; the others are read in the zone where the track was recorded,
   which you can change if the camera was set to another one. The proposed pins move along the
   track as you change the clock.
4. Check the list: each photo says whether it lands on the track or how far before, after or between
   track points it was taken. **Match within** sets how far from a track point a photo may be.
   Hover a pin on the map to see its photo beside the pointer and highlighted in the list; click the
   pin or a row to open the photo in the loupe, where the arrow keys step through the track's photos.
5. Select **Place** to save the locations. Photos that already have one keep it unless you tick
   **Also move**. **Undo** in the notification puts the previous locations back.

## Tiles and privacy

Map tiles follow Immich's map style by default (or a custom MapLibre style set by an administrator),
loaded directly by your browser. Place searches use Photon proxied through immich-edit so the
geocoder never sees your browser IP, and reverse-geocoded place names come from Immich locally. See
[Administration](../hosting/administration.md#map).
