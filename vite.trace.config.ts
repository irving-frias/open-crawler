import { defineConfig } from 'vite';
import base from './vite.config';

const logPlugin = () => {
	let isSsr = false;
	return {
		name: 'trace-context',
		enforce: 'pre',
		configResolved(config) {
			console.log(`CONFIG[${config.build.ssr ? 'SSR' : 'CLIENT'}]: resolve.conditions =`, JSON.stringify(config.resolve.conditions));
			console.log(`CONFIG[${config.build.ssr ? 'SSR' : 'CLIENT'}]: ssr.resolve.conditions =`, JSON.stringify(config.ssr?.resolve?.conditions));
		},
		buildStart(options) {
			isSsr = options.ssr ?? false;
		},
		async resolveId(source, importer) {
			if (source === 'svelte') {
				const resolved = await this.resolve(source, importer, { skipSelf: true });
				if (resolved) {
					const strip = (p) => p.replace(/^.*node_modules\.pnpm\//, '');
					console.log(`RESOLVE4[${isSsr ? 'SSR' : 'CLIENT'}]:`, JSON.stringify({ importer: strip(importer ?? ''), id: strip(resolved.id) }));
				}
			}
			return null;
		},
	};
};

export default defineConfig({
	...base,
	plugins: [...(base.plugins || []), logPlugin()],
});
