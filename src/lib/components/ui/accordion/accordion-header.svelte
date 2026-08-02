<script lang="ts">
	import { Accordion as AccordionPrimitive } from "bits-ui";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & AccordionPrimitive.HeaderProps = $props();
</script>

<AccordionPrimitive.Header
	bind:ref
	data-slot="accordion-header"
	class={cn("", className)}
	{...restProps}
>
	{@render children?.()}
</AccordionPrimitive.Header>