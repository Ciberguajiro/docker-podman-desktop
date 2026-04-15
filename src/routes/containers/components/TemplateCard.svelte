<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Rocket, Box } from "lucide-svelte";
  import type { ContainerTemplate } from "$lib/types";
  import { i18n } from "$lib/stores/i18n.svelte";

  let { template, onDeploy }: { template: ContainerTemplate, onDeploy: (t: ContainerTemplate) => void } = $props();
</script>

<Card.Root class="group hover:border-primary/50 transition-all flex flex-col h-full bg-card/50 backdrop-blur-sm">
  <Card.Header class="pb-2">
    <div class="flex items-center gap-3 mb-1">
      <div class="p-2 rounded-lg bg-primary/10 text-primary group-hover:bg-primary group-hover:text-primary-foreground transition-colors">
        <Box class="h-5 w-5" />
      </div>
      <Card.Title class="text-lg font-bold">{template.name}</Card.Title>
    </div>
    <Card.Description class="line-clamp-2 min-h-[2.5rem] text-xs">
      {template.description}
    </Card.Description>
  </Card.Header>
  <Card.Content class="pb-4 mt-auto">
    <div class="flex flex-col gap-1.5 text-[11px] text-muted-foreground">
      <div class="flex justify-between">
        <span>Image:</span>
        <span class="font-mono text-primary/80">{template.image}</span>
      </div>
      {#if template.ports}
      <div class="flex justify-between">
        <span>Ports:</span>
        <span>{template.ports}</span>
      </div>
      {/if}
    </div>
  </Card.Content>
  <Card.Footer class="pt-0 border-t border-muted/50 mt-auto bg-muted/20 rounded-b-lg">
    <Button
      variant="ghost"
      size="sm"
      class="w-full h-9 gap-2 hover:bg-primary hover:text-primary-foreground transition-all mt-3"
      onclick={() => onDeploy(template)}
    >
      <Rocket class="h-4 w-4" />
      {i18n.t('Deploy')}
    </Button>
  </Card.Footer>
</Card.Root>
