import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
  root: 'docs',
  base: './',
  build: {
    sourcemap: true,
    target: 'es2020'
  },
  plugins: [
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['editor.wasm', 'cursor_wasm.wasm'],
      manifest: {
        name: 'ElephantNote',
        short_name: 'ElephantNote',
        start_url: './index.html',
        display: 'standalone',
        background_color: '#070707',
        theme_color: '#070707',
        lang: 'fr'
      },
      workbox: {
        globPatterns: ['**/*.{js,css,html,wasm,webmanifest}']
      }
    })
  ]
});
