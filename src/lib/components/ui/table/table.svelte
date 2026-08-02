<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="table"
	class={cn("relative w-full overflow-auto", className)}
	{...restProps}
>
	<table class="w-full caption-bottom text-sm">
		{@render children?.()}
	</table>
</div>