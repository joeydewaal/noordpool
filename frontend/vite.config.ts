import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [
        tailwindcss(),
        sveltekit(),
        SvelteKitPWA({
            devOptions: {
                enabled: true
            },
            registerType: 'autoUpdate',
            manifest: {
                name: 'Noordpool',
                short_name: 'Noordpool',
                description: 'Noordpool voetbal team app',
                lang: 'nl',
                theme_color: '#1e3a5f',
                background_color: '#ffffff',
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
            '/api': 'http://localhost:3000'
        }
    },
    preview: {
        allowedHosts: true
    }
});
