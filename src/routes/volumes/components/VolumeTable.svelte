<script lang="ts">
  import type { DockerVolume } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import * as Table from "$lib/components/ui/table";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import VolumeActions from './VolumeActions.svelte';
  import { Copy, ChevronUp, ChevronDown, Database } from "lucide-svelte";

  let {
    volumes,
    sortCol,
    sortDesc,
    onSort,
    onCopy,
    onInspect,
    onRemove
  } = $props<{
    volumes: DockerVolume[];
    sortCol: string;
    sortDesc: boolean;
    onSort: (col: string) => void;
    onCopy: (text: string) => void;
    onInspect: (v: DockerVolume) => void;
    onRemove: (v: DockerVolume) => void;
  }>();
</script>

<div class="rounded-md border bg-card overflow-hidden">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[30%] cursor-pointer" onclick={() => onSort('name')}>
          <div class="flex items-center gap-1">
            {i18n.t('Name')}
            {#if sortCol === 'name'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="w-[15%] cursor-pointer" onclick={() => onSort('driver')}>
          <div class="flex items-center gap-1">
            {i18n.t('Driver')}
            {#if sortCol === 'driver'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden md:table-cell w-[30%]">{i18n.t('Mountpoint')}</Table.Head>
        <Table.Head class="hidden sm:table-cell w-[15%] cursor-pointer" onclick={() => onSort('created')}>
          <div class="flex items-center gap-1">
            {i18n.t('Created')}
            {#if sortCol === 'created'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="text-right w-[10%]">{i18n.t('Actions')}</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each volumes as v (v.name)}
        <Table.Row class="group">
          <Table.Cell>
            <div class="flex items-center gap-2">
              <Database class="h-4 w-4 text-muted-foreground" />
              <div class="font-bold truncate">{v.name}</div>
              <Button
                variant="ghost"
                size="icon"
                class="h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={() => onCopy(v.name)}
              >
                <Copy class="h-3 w-3" />
              </Button>
            </div>
          </Table.Cell>
          <Table.Cell>
            <code class="text-xs bg-muted px-1.5 py-0.5 rounded">{v.driver}</code>
          </Table.Cell>
          <Table.Cell class="hidden md:table-cell">
            <div class="text-xs text-muted-foreground truncate" title={v.mountpoint}>
              {v.mountpoint}
            </div>
          </Table.Cell>
          <Table.Cell class="hidden sm:table-cell text-xs text-muted-foreground">{v.created}</Table.Cell>
          <Table.Cell class="text-right">
            <VolumeActions
              volume={v}
              {onInspect}
              {onRemove}
            />
          </Table.Cell>
        </Table.Row>
      {/each}
      {#if volumes.length === 0}
        <Table.Row>
          <Table.Cell colspan={5} class="h-24 text-center text-muted-foreground">
            {i18n.t('NoVolumesFound') || 'No volumes found'}
          </Table.Cell>
        </Table.Row>
      {/if}
    </Table.Body>
  </Table.Root>
</div>
