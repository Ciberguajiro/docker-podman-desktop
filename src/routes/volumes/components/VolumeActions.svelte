<script lang="ts">
  import type { DockerVolume } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { Search, Trash2 } from "lucide-svelte";

  let {
    volume,
    onInspect,
    onRemove
  } = $props<{
    volume: DockerVolume;
    onInspect: (v: DockerVolume) => void;
    onRemove: (v: DockerVolume) => void;
  }>();
</script>

<div class="flex items-center justify-end gap-1">
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onInspect(volume)}>
          <Search class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Inspect')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:text-destructive" {...props} onclick={() => onRemove(volume)}>
          <Trash2 class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Remove')}</Tooltip.Content>
  </Tooltip.Root>
</div>
