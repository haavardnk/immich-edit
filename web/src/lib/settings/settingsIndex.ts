import type { SettingsPageId } from '#lib/settings/pages.ts';

export interface SettingsEntry {
  id: string;
  page: SettingsPageId;
  title: string;
  detail: string;
  keywords?: readonly string[];
  anchor?: string;
  installable?: boolean;
}

export const SETTINGS_INDEX: readonly SettingsEntry[] = [
  {
    id: 'rendering',
    page: 'preferences',
    title: 'Preview renderer',
    detail: 'Fastest, Browser, Server',
    keywords: ['webgpu', 'gpu', 'preview', 'speed'],
    anchor: 'renderer'
  },
  {
    id: 'layout',
    page: 'preferences',
    title: 'Layout',
    detail: 'Auto, Desktop, Touch',
    keywords: ['touch', 'tablet', 'phone', 'desktop'],
    anchor: 'layout'
  },
  {
    id: 'install',
    page: 'preferences',
    title: 'Install app',
    detail: 'Home Screen, Dock or app list',
    keywords: ['pwa', 'home screen', 'dock', 'standalone'],
    anchor: 'install',
    installable: true
  },
  {
    id: 'export.auto',
    page: 'preferences',
    title: 'Save exports automatically',
    detail: 'Save each export as soon as it renders',
    keywords: ['download', 'export'],
    anchor: 'save-automatically'
  },
  {
    id: 'account',
    page: 'account',
    title: 'Account',
    detail: 'Name, email, sign-in method and sign out',
    keywords: ['profile', 'email', 'sign out', 'log out', 'logout']
  },
  {
    id: 'sessions',
    page: 'account',
    title: 'Signed-in devices',
    detail: 'Revoke a session or every other session',
    keywords: ['session', 'revoke', 'browser', 'device'],
    anchor: 'sessions'
  },
  {
    id: 'appearance.mode',
    page: 'preferences',
    title: 'Mode',
    detail: 'System, Light, Dark',
    keywords: ['theme', 'dark mode', 'light mode'],
    anchor: 'mode'
  },
  {
    id: 'appearance.themes',
    page: 'preferences',
    title: 'Themes and accents',
    detail: 'A theme and an accent colour for light and for dark',
    keywords: [
      'colour',
      'color',
      'catppuccin',
      'rosé pine',
      'tokyo night',
      'gruvbox',
      'solarized',
      'nord',
      'dracula',
      'github',
      'everforest'
    ],
    anchor: 'light-theme'
  },
  {
    id: 'export-presets',
    page: 'account',
    title: 'Export presets',
    detail: 'Saved export settings',
    keywords: ['preset', 'export', 'delete'],
    anchor: 'export-presets'
  },
  {
    id: 'users',
    page: 'administration',
    title: 'Users',
    detail: 'Enable, disable or purge a user',
    keywords: ['access', 'disable', 'purge', 'admin'],
    anchor: 'users'
  },
  {
    id: 'models',
    page: 'administration',
    title: 'Models',
    detail: 'AI masks and AI Denoise models',
    keywords: ['ai', 'mask', 'subject', 'people', 'sky', 'depth', 'denoise', 'download'],
    anchor: 'models'
  },
  {
    id: 'defaults.lens',
    page: 'administration',
    title: 'Automatic lens corrections',
    detail: 'Apply matched lens profiles to raw files',
    keywords: ['lens', 'lensfun', 'profile', 'distortion', 'vignetting'],
    anchor: 'lens-auto'
  },
  {
    id: 'instance',
    page: 'administration',
    title: 'Immich instance',
    detail: 'The connected Immich server and its URL',
    keywords: ['immich', 'url', 'server', 'rebind'],
    anchor: 'instance'
  },
  {
    id: 'diagnostics.server',
    page: 'diagnostics',
    title: 'Server',
    detail: 'Version, renderer and GPU adapter',
    keywords: ['gpu', 'vulkan', 'metal', 'version'],
    anchor: 'server'
  },
  {
    id: 'diagnostics.browser',
    page: 'diagnostics',
    title: 'Browser renderer',
    detail: 'WebGPU adapter and status',
    keywords: ['webgpu', 'gpu', 'render'],
    anchor: 'browser-renderer'
  },
  {
    id: 'diagnostics.latency',
    page: 'diagnostics',
    title: 'Render latency',
    detail: 'Recent preview and export timings',
    keywords: ['slow', 'speed', 'timing', 'render'],
    anchor: 'render-latency'
  },
  {
    id: 'diagnostics.bundle',
    page: 'diagnostics',
    title: 'Copy support bundle',
    detail: 'A diagnostics block for a bug report',
    keywords: ['bug', 'report', 'support']
  }
];
