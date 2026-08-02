<script lang="ts">
	import { Tooltip as TooltipPrimitive } from "bits-ui";

	let {
		children,
		delayDuration = 700,
		disableCloseOnTriggerClick = false,
		disableHoverableContent = false,
		disabled = false,
		ignoreNonKeyboardFocus = false,
		skipDelayDuration = 300,
		...restProps
	}: TooltipPrimitive.ProviderProps = $props();
</script>

<TooltipPrimitive.Provider
	{delayDuration}
	{disableCloseOnTriggerClick}
	{disableHoverableContent}
	{disabled}
	{ignoreNonKeyboardFocus}
	{skipDelayDuration}
	{...restProps}
>
	{@render children?.()}
</TooltipPrimitive.Provider>