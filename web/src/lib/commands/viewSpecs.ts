import type { Command, IconCommand } from '#lib/commands/types.ts';
import { mdiFullscreen, mdiInformationOutline, mdiTriangleOutline } from '@mdi/js';

type ViewSpec = Pick<IconCommand, 'icon'> & Required<Pick<Command, 'label' | 'bind'>>;

export const infoView: ViewSpec = {
  label: 'Info',
  icon: mdiInformationOutline,
  bind: 'toggleInfo'
};

export const clipWarnView: ViewSpec = {
  label: 'Clipping overlay',
  icon: mdiTriangleOutline,
  bind: 'clipWarn'
};

export const fullscreenView: ViewSpec = {
  label: 'Fullscreen',
  icon: mdiFullscreen,
  bind: 'fullscreen'
};
