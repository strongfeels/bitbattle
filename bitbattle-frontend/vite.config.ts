import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
    plugins: [react()],
    resolve: {
        extensions: ['.js', '.ts', '.jsx', '.tsx']
    },
    build: {
        // Enable source maps for production debugging
        sourcemap: true,
        // Code splitting configuration
        rollupOptions: {
            output: {
                manualChunks(id) {
                    // Separate vendor chunks for better caching
                    if (id.includes('node_modules')) {
                        if (id.includes('react') || id.includes('react-dom')) {
                            return 'react-vendor';
                        }
                        if (id.includes('codemirror') || id.includes('@uiw')) {
                            return 'codemirror';
                        }
                    }
                },
            },
        },
        // Increase chunk size warning limit (CodeMirror is large)
        chunkSizeWarningLimit: 600,
    },
    // Preview server configuration
    preview: {
        port: 5173,
        host: true,
    },
    // Dev server configuration
    server: {
        port: 5173,
        host: true,
    },
})
