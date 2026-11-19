import { JOB_KINDS } from '#lib/api/jobs.ts';
import {
  mdiCloudUploadOutline,
  mdiCogOutline,
  mdiContentPaste,
  mdiFolderZipOutline,
  mdiLayersTripleOutline,
  mdiRestore
} from '@mdi/js';

const KIND_ICONS: Record<string, string> = {
  [JOB_KINDS.exportImmich]: mdiCloudUploadOutline,
  [JOB_KINDS.downloadZip]: mdiFolderZipOutline,
  [JOB_KINDS.applyPreset]: mdiLayersTripleOutline,
  [JOB_KINDS.pasteEdits]: mdiContentPaste,
  [JOB_KINDS.resetEdits]: mdiRestore
};

export function jobKindIcon(kind: string): string {
  return KIND_ICONS[kind] ?? mdiCogOutline;
}
