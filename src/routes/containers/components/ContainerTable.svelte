<script lang="ts">
  import type { DockerContainer } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import * as Table from "$lib/components/ui/table";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import StatusBadge from "$lib/components/ui/StatusBadge.svelte";
  import ContainerActions from './ContainerActions.svelte';
  import { Copy, ChevronUp, ChevronDown } from "lucide-svelte";

  let {
    containers,
    sortCol,
    sortDesc,
    onSort,
    onCopy,
    onStart,
    onStop,
    onExec,
    onFiles,
    onLogs,
    onExport,
    onInspect,
    onRemove
  } = $props<{
    containers: DockerContainer[];
    sortCol: string;
    sortDesc: boolean;
    onSort: (col: string) => void;
    onCopy: (text: string) => void;
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

<div class="rounded-md border bg-card">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="cursor-pointer" onclick={() => onSort('name')}>
          <div class="flex items-center gap-1">
            {i18n.t('Name')}
            {#if sortCol === 'name'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden md:table-cell cursor-pointer" onclick={() => onSort('image')}>
          <div class="flex items-center gap-1">
            {i18n.t('Image')}
            {#if sortCol === 'image'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="text-center">{i18n.t('Ports')}</Table.Head>
        <Table.Head class="text-center cursor-pointer" onclick={() => onSort('state')}>
          <div class="flex items-center justify-center gap-1">
            {i18n.t('State')}
            {#if sortCol === 'state'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="text-right">{i18n.t('Actions')}</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each containers as c (c.id)}
        <Table.Row class="group">
          <Table.Cell>
            <div class="font-medium truncate max-w-[200px]">{c.name}</div>
            <div class="flex items-center gap-1.5 mt-0.5">
              <code class="text-[10px] text-muted-foreground font-mono">{c.id.slice(0, 12)}</code>
              <Button
                variant="ghost"
                size="icon"
                class="h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={() => onCopy(c.id)}
              >
                <Copy class="h-3 w-3" />
              </Button>
            </div>
          </Table.Cell>
          <Table.Cell class="hidden md:table-cell">
            <div class="text-xs text-muted-foreground truncate max-w-[200px] italic" title={c.image}>
              {c.compose_service || c.image}
            </div>
          </Table.Cell>
          <Table.Cell class="text-center">
            {#if c.ports.length > 0}
              <div class="flex flex-wrap justify-center gap-1">
                {#each c.ports as port}
                  <Badge variant="outline" class="text-[10px] px-1.5 h-4">{port}</Badge>
                {/each}
              </div>
            {:else}
              <span class="text-[10px] text-muted-foreground">{i18n.t('NoPorts')}</span>
            {/if}
          </Table.Cell>
          <Table.Cell class="text-center">
            <StatusBadge status={c.state} />
          </Table.Cell>
          <Table.Cell class="text-right">
            <ContainerActions
              container={c}
              {onStart}
              {onStop}
              {onExec}
              {onFiles}
              {onLogs}
              {onExport}
              {onInspect}
              {onRemove}
            />
          </Table.Cell>
        </Table.Row>
      {/each}
      {#if containers.length === 0}
        <Table.Row>
          <Table.Cell colspan={5} class="h-24 text-center text-muted-foreground">
            {i18n.t('NoContainersFound')}
          </Table.Cell>
        </Table.Row>
      {/if}
    </Table.Body>
  </Table.Root>
</div>
