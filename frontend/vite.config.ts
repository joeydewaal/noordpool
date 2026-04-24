import tailwindcss from "@tailwindcss/vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { SvelteKitPWA } from "@vite-pwa/sveltekit";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },
  plugins: [
    tailwindcss(),
    sveltekit(),
    SvelteKitPWA({
      devOptions: {
        enabled: true,
        type: "module",
      },
      // injectManifest mode: ship a custom service worker so we can
      // handle Web Push (`push`, `notificationclick`) on top of the
      // Workbox precache.
      strategies: "injectManifest",
      srcDir: "src",
      filename: "service-worker.ts",
      registerType: "autoUpdate",
      injectManifest: {
        globPatterns: ["client/**/*.{js,css,ico,png,svg,webp,woff,woff2}"],
        globIgnores: ["client/_app/immutable/nodes/**"],
      },
      manifest: {
        name: "Noordpool",
        short_name: "Noordpool",
        description: "Noordpool voetbal team app",
        lang: "nl",
        theme_color: "#0a0a0a",
        background_color: "#0a0a0a",
        display: "standalone",
        icons: [
          { src: "/icons/icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "/icons/icon-512.png", sizes: "512x512", type: "image/png" },
          { src: "/icons/icon-512.webp", sizes: "512x512", type: "image/webp" },
        ],
      },
    }),
  ],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        ws: true,
      },
      "/avatars": {
        target: "http://localhost:3000",
      },
    },
    allowedHosts: true,
  },
  preview: {
    allowedHosts: true,
  },
});
