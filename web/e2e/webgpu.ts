const swiftshader =
  process.platform === 'linux' || process.env.WEBGPU_SWIFTSHADER === '1'
    ? [
        '--enable-features=Vulkan',
        '--use-vulkan=swiftshader',
        '--use-webgpu-adapter=swiftshader',
        '--disable-vulkan-surface'
      ]
    : [];

export const webgpu = {
  channel: 'chromium',
  headless: process.platform !== 'linux',
  launchOptions: { args: ['--enable-unsafe-webgpu', ...swiftshader] }
};
