import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.test.ts"],
    globals: true,
    browser: {
      enabled: false,
    },
  },
  resolve: {
    conditions: ["browser"],
    alias: {
      $lib: "/home/joey/dev/noordpool/frontend/src/lib",
      $app: "/home/joey/dev/noordpool/frontend/src/test/mocks/app",
    },
  },
});
