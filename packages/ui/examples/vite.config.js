import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    root: __dirname,
    plugins: [
        solid(), tailwindcss()
    ],
    server: { port: 5173 }
});
