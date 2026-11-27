// @vitest-environment jsdom
import { parseTrackFile, trackFormat } from '#lib/geotag/trackFile.ts';
import { describe, expect, it } from 'vitest';

const GARMIN = `<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Garmin" xmlns="http://www.topografix.com/GPX/1/1">
  <metadata><name>Not the track</name></metadata>
  <wpt lat="61" lon="7"><time>2026-10-31T07:30:00Z</time><name>Cabin</name></wpt>
  <trk>
    <name>Hardangervidda &amp; Trolltunga</name>
    <trkseg>
      <trkpt lat="60.1240" lon="6.7400"><ele>1100</ele><time>2026-10-31T08:12:05Z</time></trkpt>
      <trkpt lat="60.1250" lon="6.7410"><time>2026-10-31T08:12:00Z</time></trkpt>
      <trkpt lat="60.1260" lon="6.7420"></trkpt>
    </trkseg>
    <trkseg>
      <trkpt lat="60.2000" lon="6.8000"><time>2026-10-31T09:00:00.500+01:00</time></trkpt>
      <trkpt lat="60.2010" lon="6.8010"><time>2026-10-31T09:01:00+01:00</time></trkpt>
    </trkseg>
  </trk>
  <rte><rtept lat="5" lon="5"><time>2026-10-31T09:00:00Z</time></rtept></rte>
</gpx>`;

const TCX = `<?xml version="1.0"?>
<TrainingCenterDatabase xmlns="http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2">
  <Activities><Activity Sport="Hiking"><Lap><Track>
    <Trackpoint><Time>2026-10-31T08:00:00Z</Time><Position>
      <LatitudeDegrees>60.1</LatitudeDegrees><LongitudeDegrees>6.7</LongitudeDegrees>
    </Position></Trackpoint>
    <Trackpoint><Time>2026-10-31T08:00:10Z</Time><Position>
      <LatitudeDegrees>60.2</LatitudeDegrees><LongitudeDegrees>6.8</LongitudeDegrees>
    </Position></Trackpoint>
  </Track></Lap></Activity></Activities>
</TrainingCenterDatabase>`;

const KML = `<?xml version="1.0"?>
<kml xmlns="http://www.opengis.net/kml/2.2" xmlns:gx="http://www.google.com/kml/ext/2.2">
  <Document><Placemark><name>Walk</name><gx:Track>
    <when>2026-10-31T08:00:00Z</when><when>2026-10-31T08:05:00Z</when>
    <gx:coord>6.7 60.1 0</gx:coord><gx:coord>6.8 60.2 0</gx:coord>
  </gx:Track></Placemark></Document>
</kml>`;

describe('track files', () => {
  it('knows GPX, KML and TCX by name', () => {
    expect(trackFormat('Walk.GPX')).toBe('gpx');
    expect(trackFormat('ride.tcx')).toBe('tcx');
    expect(trackFormat('trip.kml')).toBe('kml');
    expect(trackFormat('photo.jpg')).toBeNull();
  });

  it('reads GPX tracks and segments in time order and skips routes and untimed points', () => {
    const { lines, untimed } = parseTrackFile(GARMIN, 'gpx');
    expect(untimed).toBe(1);
    expect(lines).toHaveLength(1);
    expect(lines[0]?.name).toBe('Hardangervidda & Trolltunga');
    expect(lines[0]?.segments.map((points) => points.length)).toEqual([2, 2]);
    expect(lines[0]?.segments[0]?.[0]).toEqual({
      lat: 60.125,
      lon: 6.741,
      time: Date.parse('2026-10-31T08:12:00Z')
    });
    expect(lines[0]?.segments[1]?.[0]?.time).toBe(Date.parse('2026-10-31T08:00:00.500Z'));
  });

  it('keeps each GPX time on its own point when one in the middle has none', () => {
    const source = `<gpx version="1.1" xmlns="http://www.topografix.com/GPX/1/1"><trk><trkseg>
      <trkpt lat="1" lon="1"><time>2026-10-31T08:00:00Z</time></trkpt>
      <trkpt lat="2" lon="2"></trkpt>
      <trkpt lat="3" lon="3"><time>2026-10-31T08:02:00Z</time></trkpt>
    </trkseg></trk></gpx>`;
    const { lines, untimed } = parseTrackFile(source, 'gpx');
    expect(untimed).toBe(1);
    expect(lines[0]?.segments[0]).toEqual([
      { lat: 1, lon: 1, time: Date.parse('2026-10-31T08:00:00Z') },
      { lat: 3, lon: 3, time: Date.parse('2026-10-31T08:02:00Z') }
    ]);
  });

  it('reads TCX activities and KML tracks', () => {
    expect(parseTrackFile(TCX, 'tcx').lines[0]?.segments[0]).toEqual([
      { lat: 60.1, lon: 6.7, time: Date.parse('2026-10-31T08:00:00Z') },
      { lat: 60.2, lon: 6.8, time: Date.parse('2026-10-31T08:00:10Z') }
    ]);
    const walk = parseTrackFile(KML, 'kml').lines[0];
    expect(walk?.name).toBe('Walk');
    expect(walk?.segments[0]?.map((point) => point.time)).toEqual([
      Date.parse('2026-10-31T08:00:00Z'),
      Date.parse('2026-10-31T08:05:00Z')
    ]);
  });

  it('tells UTC times from times with their own offset', () => {
    const at = (time: string) =>
      parseTrackFile(
        `<gpx version="1.1" xmlns="http://www.topografix.com/GPX/1/1"><trk><trkseg>
          <trkpt lat="1" lon="1"><time>${time}</time></trkpt>
          <trkpt lat="1" lon="2"><time>${time}</time></trkpt>
        </trkseg></trk></gpx>`,
        'gpx'
      ).lines[0]?.utc;
    expect(at('2026-10-31T08:00:00Z')).toBe(true);
    expect(at('2026-10-31T08:00:00')).toBe(true);
    expect(at('2026-10-31T08:00:00+00:00')).toBe(true);
    expect(at('2026-10-31T11:00:00+03:00')).toBe(false);
  });

  it('finds nothing in a broken or empty file', () => {
    expect(parseTrackFile('not xml <', 'gpx')).toEqual({ lines: [], untimed: 0 });
    expect(parseTrackFile('<gpx><wpt lat="1" lon="2"/></gpx>', 'gpx')).toEqual({
      lines: [],
      untimed: 0
    });
  });
});
