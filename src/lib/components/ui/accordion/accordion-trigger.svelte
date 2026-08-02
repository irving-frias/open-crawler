<script lang="ts">
	import { Accordion as AccordionPrimitive } from "bits-ui";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLButtonElement>> & AccordionPrimitive.TriggerProps = $props();
</script>

<AccordionPrimitive.Trigger
	bind:ref
	data-slot="accordion-trigger"
	class={cn(
		"flex flex-1 items-center justify-between py-4 text-sm font-medium transition-all hover:underline [&[data-state=open]>svg]:rotate-180",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</AccordionPrimitive.Trigger>