export interface MaskHandleProps {
  cx: number;
  cy: number;
  r: number;
  hit: number;
  label: string;
  cursor: string;
  fill: string;
  fillOpacity?: number;
  stroke: string;
  strokeWidth: number;
  onpointerdown: (e: PointerEvent) => void;
  ondblclick?: (e: MouseEvent) => void;
}

export type MaskHandleSpec = Omit<MaskHandleProps, 'hit'> & { hit?: number };
