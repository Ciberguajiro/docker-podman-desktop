<script lang="ts">
  import type { DockerContainer } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { Button } from "$lib/components/ui/button";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    Play,
    Square,
    Terminal,
    Folder,
    ScrollText,
    ExternalLink,
    Search,
    Trash2
  } from "lucide-svelte";

  let {
    container,
    onStart,
    onStop,
    onExec,
    onFiles,
    onLogs,
    onExport,
    onInspect,
    onRemove
  } = $props<{
    container: DockerContainer;
    onStart: (c: DockerContainer) => void;
    onStop: (c: DockerContainer) => void;
    onExec: (c: DockerContainer) => void;
    onFiles: (c: DockerContainer) => void;
    onLogs: (c: DockerContainer) => void;
    onExport: (c: DockerContainer) => void;
    onInspect: (c: DockerContainer) => void;
    onRemove: (c: DockerContainer) => void;
  }>();
</script>

<div class="flex items-center justify-end gap-1">
  {#if container.state !== 'running'}
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button variant="ghost" size="icon" class="h-8 w-8 text-green-600" {...props} onclick={() => onStart(container)}>
            <Play class="h-4 w-4" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>{i18n.t('Run') || 'Start'}</Tooltip.Content>
    </Tooltip.Root>
  {:else}
    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button variant="ghost" size="icon" class="h-8 w-8 text-amber-600" {...props} onclick={() => onStop(container)}>
            <Square class="h-4 w-4" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>{i18n.t('Close') || 'Stop'}</Tooltip.Content>
    </Tooltip.Root>
  {/if}

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} disabled={container.state !== 'running'} onclick={() => onExec(container)}>
          <Terminal class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Exec')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} disabled={container.state !== 'running'} onclick={() => onFiles(container)}>
          <Folder class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Files')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onLogs(container)}>
          <ScrollText class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Logs')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onExport(container)}>
          <ExternalLink class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Export')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8" {...props} onclick={() => onInspect(container)}>
          <Search class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Inspect')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:text-destructive" {...props} onclick={() => onRemove(container)}>
          <Trash2 class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Remove')}</Tooltip.Content>
  </Tooltip.Root>
</div>
