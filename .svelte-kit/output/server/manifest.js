export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.png"]),
	mimeTypes: {".png":"image/png"},
	_: {
		client: {start:"_app/immutable/entry/start.BT4mBa23.js",app:"_app/immutable/entry/app.BkXV3YYB.js",imports:["_app/immutable/entry/start.BT4mBa23.js","_app/immutable/chunks/B9rB3RhD.js","_app/immutable/chunks/DZWouQ5J.js","_app/immutable/chunks/CGZWmbXy.js","_app/immutable/chunks/DrYs8Tfo.js","_app/immutable/entry/app.BkXV3YYB.js","_app/immutable/chunks/DrYs8Tfo.js","_app/immutable/chunks/CGZWmbXy.js","_app/immutable/chunks/DZWouQ5J.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js'))
		],
		remotes: {
			
		},
		routes: [
			
		],
		prerendered_routes: new Set(["/"]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
