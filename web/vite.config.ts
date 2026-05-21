import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

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
	}
});

