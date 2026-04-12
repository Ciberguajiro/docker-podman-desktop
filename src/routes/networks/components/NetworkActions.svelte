<script lang="ts">
  import type { DockerNetwork } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { Search, Trash2 } from "lucide-svelte";

  let {
    network,
    onInspect,
    onRemove
  } = $props<{
    network: DockerNetwork;
    onInspect: (n: DockerNetwork) => void;
    onRemove: (n: DockerNetwork) => void;
  }>();
</script>

<div class="flex items-center justify-end gap-1">
  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} onclick={() => onInspect(network)}>
          <Search class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Inspect')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:text-destructive" builders={[builder]} onclick={() => onRemove(network)}>
          <Trash2 class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Remove')}</Tooltip.Content>
  </Tooltip.Root>
</div>
