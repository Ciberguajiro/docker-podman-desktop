<script lang="ts">
  import type { DockerImage } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    Play,
    Tag,
    Search,
    History,
    Trash2
  } from "lucide-svelte";

  let {
    image,
    onRun,
    onTag,
    onInspect,
    onHistory,
    onRemove
  } = $props<{
    image: DockerImage;
    onRun: (img: DockerImage) => void;
    onTag: (img: DockerImage) => void;
    onInspect: (img: DockerImage) => void;
    onHistory: (img: DockerImage) => void;
    onRemove: (img: DockerImage) => void;
  }>();
</script>

<div class="flex items-center justify-end gap-1">
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-green-600" {...props} onclick={() => onRun(image)}>
          <Play class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Run')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onTag(image)}>
          <Tag class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Tag')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onInspect(image)}>
          <Search class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Inspect')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onHistory(image)}>
          <History class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('History')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:text-destructive" {...props} onclick={() => onRemove(image)}>
          <Trash2 class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Remove')}</Tooltip.Content>
  </Tooltip.Root>
</div>
