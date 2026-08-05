import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

const backend = process.env.IMMICH_EDIT_BACKEND ?? 'http://127.0.0.1:8088';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    proxy: {
      '/api': {
        target: backend,
        changeOrigin: false
      }
    }
  },
  test: {
    include: ['src/**/*.{test,spec}.ts'],
    exclude: ['e2e/**'],
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      include: ['src/lib/**']
    }
  }
});
