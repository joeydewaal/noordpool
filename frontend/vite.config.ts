import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'jsdom',
        setupFiles: ['./src/test-setup.ts'],
    },
    plugins: [
        tailwindcss(),
        sveltekit(),
        SvelteKitPWA({
            devOptions: {
                enabled: true
            },
            registerType: 'autoUpdate',
            workbox: {
                runtimeCaching: [
                    {
                        urlPattern: /^.*\/api\/(games|players|stats)/,
                        handler: 'StaleWhileRevalidate',
                        options: {
                            cacheName: 'api-cache',
                            expiration: {
                                maxEntries: 50,
                                maxAgeSeconds: 60 * 60 * 24,
                            },
                        },
                    },
                ],
            },
            manifest: {
                name: 'Noordpool',
                short_name: 'Noordpool',
                description: 'Noordpool voetbal team app',
                lang: 'nl',
                theme_color: '#0a0a0a',
                background_color: '#0a0a0a',
                display: 'standalone',
                icons: [
                    { src: '/icons/icon-192.png', sizes: '192x192', type: 'image/png' },
                    { src: '/icons/icon-512.png', sizes: '512x512', type: 'image/png' }
                ]
            },
        })
    ],
    server: {
        proxy: {
            '/api': 'http://localhost:3000',
        },
        allowedHosts: true,
    },
    preview: {
        allowedHosts: true
    }
});
