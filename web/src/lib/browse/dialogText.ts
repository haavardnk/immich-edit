export type DialogTextPart = string | { strong: string };

export interface DialogText {
  title: string;
  confirm: string;
  lines: DialogTextPart[][];
}
