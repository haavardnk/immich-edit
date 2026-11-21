export interface MenuActionInfo {
  label: string;
  icon: string;
  danger: boolean;
}

export type MenuGroups<A extends string> = readonly (readonly A[])[];
