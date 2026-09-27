import { bandsAllZero, bwIsNeutral, colorGradeIsZero, dcpIsDefault, lut3dIsActive } from './color';
import { flatOpsAreIdentity } from './flatOps';
import { geometryIsIdentity } from './geometry';
import type { Edits } from './model';
import { curvesEditsIsIdentity } from './tone';

export function isIdentity(e: Edits): boolean {
  return dcpIsDefault(e.color.dcp) && isNonGeometryIdentity(e) && geometryIsIdentity(e.geometry);
}

export function isNonGeometryIdentity(e: Edits): boolean {
  return isDevelopIdentity(e) && e.masks.length === 0 && e.retouch.length === 0;
}

export function isDevelopIdentity(e: Edits): boolean {
  return (
    flatOpsAreIdentity(e) &&
    curvesEditsIsIdentity(e.basic.curves) &&
    bandsAllZero(e.color.hsl.bands) &&
    colorGradeIsZero(e.color.color_grade) &&
    !lut3dIsActive(e.color.lut_3d) &&
    bwIsNeutral(e.color.bw)
  );
}
