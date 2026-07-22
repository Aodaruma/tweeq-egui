import vue from '@vitejs/plugin-vue'
import {resolve} from 'path'
import {defineConfig} from 'vite'
import glsl from 'vite-plugin-glsl'

// Standalone Vue source-of-truth used while validating the egui port. It is a
// separate app build so the published Vue library bundle remains unchanged.
export default defineConfig({
	plugins: [glsl(), vue()],
	build: {
		outDir: 'dist/reference',
		rollupOptions: {
			input: resolve(__dirname, 'port-reference.html'),
		},
	},
	ssr: {
		noExternal: ['@baku89/pave'],
		external: ['paper', 'paper-jsdom-canvas'],
	},
	define: {
		'process.env.PROMISE_QUEUE_COVERAGE': false,
	},
})
