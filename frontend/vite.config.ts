import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    outDir: '../static',
    emptyOutDir: false, // Don't delete index.html!
    lib: {
      entry: './src/main.ts',
      formats: ['iife'],
      name: 'app',
      fileName: () => 'frontend.js'
    }
  }
});