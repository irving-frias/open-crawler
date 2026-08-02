<script lang="ts">
	import { Accordion as AccordionPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";
	import type { Snippet } from "svelte";

	let {
		class: className,
		child,
		children,
		...restProps
	}: AccordionPrimitive.ContentProps & { child?: Snippet; children?: Snippet } = $props();
</script>

<AccordionPrimitive.Content
	data-slot="accordion-content"
	class={cn(
		"overflow-hidden text-sm data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down",
		className
	)}
	{...restProps}
>
	<div class="pb-4 pt-0">
		{@render child?.()}
		{@render children?.()}
	</div>
</AccordionPrimitive.Content>