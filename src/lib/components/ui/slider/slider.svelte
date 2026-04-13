<script lang="ts">
	import { Slider as SliderPrimitive } from "bits-ui";
	import { cn } from "$lib/utils";

	let {
		ref = $bindable(null),
		value = $bindable(),
		type = "multiple",
		class: className,
		...restProps
	}: any = $props();

  let thumbs = $derived(Array.isArray(value) ? value : [value]);
</script>

<SliderPrimitive.Root
	bind:ref
	bind:value
	{type}
	class={cn(
		"relative flex w-full touch-none select-none items-center",
		className
	)}
	{...restProps}
>
	{#snippet children()}
		<span
			class="bg-secondary relative h-1.5 w-full grow overflow-hidden rounded-full"
		>
			<SliderPrimitive.Range class="bg-primary absolute h-full" />
		</span>
		{#each thumbs as _, i}
			<SliderPrimitive.Thumb
        index={i}
				class="border-primary bg-background focus-visible:ring-ring block size-4 rounded-full border shadow-sm transition-colors focus-visible:ring-1 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50"
			/>
		{/each}
	{/snippet}
</SliderPrimitive.Root>
