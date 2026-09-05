import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  // Fixed dev port so the Tauri dev launcher can reliably free it. 1440 keeps
  // Mog Studio clear of the sibling dev servers (MusicPlayer 1420, FasterDB 1430).
  server: {
    port: 1440,
    strictPort: true,
  },
})
