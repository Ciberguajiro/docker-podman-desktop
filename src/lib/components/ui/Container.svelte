<script lang="ts">
  import { cn } from "$lib/utils";

  let { children, class: className } = $props<{
    children: any;
    class?: string;
  }>();
</script>

<div class={cn("container mx-auto p-4 md:p-6 space-y-6", className)}>
  {@render children()}
</div>
