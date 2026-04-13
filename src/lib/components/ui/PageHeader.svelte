<script lang="ts">
  import { cn } from "$lib/utils";
  import { Separator } from "$lib/components/ui/separator";

  let { title, description, icon: Icon, children, class: className } = $props<{
    title: string;
    description?: string;
    icon?: any;
    children?: any;
    class?: string;
  }>();
</script>

<div class={cn("flex flex-col gap-4 mb-6", className)}>
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      {#if Icon}
        <div class="p-2 bg-primary/10 rounded-lg">
          <Icon class="w-6 h-6 text-primary" />
        </div>
      {/if}
      <div>
        <h1 class="text-2xl font-bold tracking-tight">{title}</h1>
        {#if description}
          <p class="text-sm text-muted-foreground">{description}</p>
        {/if}
      </div>
    </div>
    {#if children}
      <div class="flex items-center gap-2">
        {@render children()}
      </div>
    {/if}
  </div>
  <Separator />
</div>
