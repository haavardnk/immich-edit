export interface SectionRow {
  id: string;
  label: string;
  hidden: boolean;
}

export interface SectionGroup {
  id: string;
  label?: string;
  rows: SectionRow[];
}

export interface LockedSection {
  label: string;
  note: string;
}

export type SectionListVariant = 'menu' | 'sheet';
