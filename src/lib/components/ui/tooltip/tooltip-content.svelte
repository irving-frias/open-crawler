<script lang="ts">
	import { Tooltip as TooltipPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";
	import TooltipPortal from "./tooltip-portal.svelte";
	import type { Snippet } from "svelte";

	let {
		class: className,
		sideOffset = 4,
		portalProps,
		child,
		...restProps
	}: TooltipPrimitive.ContentProps & {
		portalProps?: { children?: Snippet } & Parameters<typeof TooltipPortal>[0];
		child?: Snippet;
	} = $props();
</script>

{#if portalProps}
	<TooltipPortal {...portalProps}>
		<TooltipPrimitive.Content
			data-slot="tooltip-content"
			{sideOffset}
			class={cn(
				"z-50 overflow-hidden rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md ring-1 ring-foreground/5 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 origin-(--transform-origin) w-fit max-w-(--radix-tooltip-trigger-width) pointer-events-none",
				className
			)}
			{...restProps}
		>
			{@render child?.()}
		</TooltipPrimitive.Content>
	</TooltipPortal>
{:else}
	<TooltipPrimitive.Content
		data-slot="tooltip-content"
		{sideOffset}
		class={cn(
			"z-50 overflow-hidden rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md ring-1 ring-foreground/5 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 origin-(--transform-origin) w-fit max-w-(--radix-tooltip-trigger-width) pointer-events-none",
			className
		)}
		{...restProps}
	>
		{@render child?.()}
	</TooltipPrimitive.Content>
{/if}