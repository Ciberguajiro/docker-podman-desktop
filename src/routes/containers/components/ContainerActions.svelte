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
    onStart: (id: string) => void;
    onStop: (id: string) => void;
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
      <Tooltip.Trigger asChild>
        {#snippet children({ builder })}
          <Button variant="ghost" size="icon" class="h-8 w-8 text-green-600" builders={[builder]} onclick={() => onStart(container.id)}>
            <Play class="h-4 w-4" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>{i18n.t('Start')}</Tooltip.Content>
    </Tooltip.Root>
  {:else}
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        {#snippet children({ builder })}
          <Button variant="ghost" size="icon" class="h-8 w-8 text-amber-600" builders={[builder]} onclick={() => onStop(container.id)}>
            <Square class="h-4 w-4" />
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content>{i18n.t('Stop')}</Tooltip.Content>
    </Tooltip.Root>
  {/if}

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} disabled={container.state !== 'running'} onclick={() => onExec(container)}>
          <Terminal class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Exec')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} disabled={container.state !== 'running'} onclick={() => onFiles(container)}>
          <Folder class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Files')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} onclick={() => onLogs(container)}>
          <ScrollText class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Logs')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} onclick={() => onExport(container)}>
          <ExternalLink class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Export')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8" builders={[builder]} onclick={() => onInspect(container)}>
          <Search class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Inspect')}</Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger asChild>
      {#snippet children({ builder })}
        <Button variant="ghost" size="icon" class="h-8 w-8 text-destructive hover:text-destructive" builders={[builder]} onclick={() => onRemove(container)}>
          <Trash2 class="h-4 w-4" />
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{i18n.t('Remove')}</Tooltip.Content>
  </Tooltip.Root>
</div>
