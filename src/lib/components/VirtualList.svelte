<script lang="ts" generics="T">
  import { onMount } from 'svelte';

  let { items, itemHeight = 60, getKey = (item: T, index: number) => index, children }: {
    items: T[],
    itemHeight?: number,
    getKey?: (item: T, index: number) => any,
    children: any // Svelte 5 snippet
  } = $props();

  let containerHeight = $state(0);
  let scrollTop = $state(0);
  let container: HTMLDivElement | undefined = $state();

  const visibleCount = $derived(Math.ceil(containerHeight / itemHeight) + 5);
  const startIndex = $derived(Math.max(0, Math.floor(scrollTop / itemHeight) - 2));
  const endIndex = $derived(Math.min(items.length, startIndex + visibleCount));

  const visibleItems = $derived(items.slice(startIndex, endIndex).map((item, i) => ({
    item,
    index: startIndex + i
  })));

  const totalHeight = $derived(items.length * itemHeight);
  const offsetY = $derived(startIndex * itemHeight);

  function handleScroll(e: Event) {
    scrollTop = (e.target as HTMLDivElement).scrollTop;
  }

  onMount(() => {
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight = entry.contentRect.height;
      }
    });

    observer.observe(container);
    containerHeight = container.offsetHeight;

    return () => observer.disconnect();
  });
</script>

<div
  bind:this={container}
  class="overflow-auto relative h-full w-full"
  onscroll={handleScroll}
>
  <div style="height: {totalHeight}px; width: 100%; pointer-events: none;"></div>
  <div
    class="absolute top-0 left-0 w-full"
    style="transform: translateY({offsetY}px);"
  >
    {#each visibleItems as { item, index } (getKey(item, index))}
      <div style="height: {itemHeight}px; overflow: hidden;">
        {@render children(item)}
      </div>
    {/each}
  </div>
</div>
