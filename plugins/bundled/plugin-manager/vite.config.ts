import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    plugins: [tailwindcss(), solid()],
    server: { port: 5174, strictPort: true },
    build: { outDir: 'dist' },
});
