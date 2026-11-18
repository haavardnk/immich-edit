import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';
import type { Plugin } from 'vite';
import { defineConfig } from 'vitest/config';

const scaleAwareMerge = fileURLToPath(new URL('./src/lib/theme/tailwindMerge.ts', import.meta.url));

const immichUiMerge: Plugin = {
  name: 'immich-ui-merge',
  enforce: 'pre',
  resolveId(source, importer) {
    if (source === 'tailwind-merge' && importer?.includes('/@immich/ui/')) return scaleAwareMerge;
  }
};

const backend = process.env.IMMICH_EDIT_BACKEND ?? 'http://127.0.0.1:8088';

export default defineConfig({
  plugins: [
    immichUiMerge,
    tailwindcss(),
    sveltekit({
      compilerOptions: {
        runes: ({ filename }) =>
          filename.split(/[/\\]/).includes('node_modules') ? undefined : true
      },
      adapter: adapter({ fallback: '200.html' })
    })
  ],
  optimizeDeps: {
    rolldownOptions: { plugins: [immichUiMerge] }
  },
  worker: {
    format: 'es'
  },
  server: {
    proxy: {
      '/api': {
        target: backend,
        changeOrigin: false
      }
    }
  },
  test: {
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      include: ['src/lib/**']
    },
    projects: [
      {
        extends: true,
        test: {
          name: 'unit',
          include: ['src/**/*.{test,spec}.ts'],
          exclude: ['e2e/**', 'src/**/*.component.test.ts'],
          environment: 'node'
        }
      },
      {
        extends: true,
        resolve: { conditions: ['browser'] },
        test: {
          name: 'component',
          include: ['src/**/*.component.test.ts'],
          environment: 'jsdom',
          setupFiles: ['./vitest-setup-client.ts']
        }
      }
    ]
  }
});
