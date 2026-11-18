import { toolRange } from './range';

export const BRUSH_SIZE = toolRange(0.005, 0.5, 0.005, 0.08, { keyStep: 0.01 });

export const BRUSH_HARDNESS = toolRange(0, 1, 0.01, 0.5, { keyStep: 0.1 });

export const BRUSH_FLOW = toolRange(0.01, 1, 0.01, 0.8);

export const RETOUCH_SIZE = toolRange(0.005, 0.3, 0.005, 0.05);

export const RETOUCH_HARDNESS = toolRange(0, 1, 0.01, 0.5, { keyStep: 0.1 });

export const RETOUCH_OPACITY = toolRange(0.05, 1, 0.01, 1);
