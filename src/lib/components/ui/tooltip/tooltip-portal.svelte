<script lang="ts">
	import { Portal } from "bits-ui";
	import type { Snippet } from "svelte";

	let {
		children,
		...restProps
	}: { children?: Snippet } & Parameters<typeof Portal>[0] = $props();
</script>

<Portal {...restProps}>
	{@render children?.()}
</Portal>