import {createPinia} from 'pinia'
import {createApp} from 'vue'

async function start() {
	try {
		const {default: PortReference} = await import('./PortReference.vue')
		createApp(PortReference).use(createPinia()).mount('#app')
	} catch (error) {
		const message = error instanceof Error ? error.stack : String(error)
		document.querySelector('#app')?.replaceChildren(message ?? 'Unknown error')
		throw error
	}
}

void start()
