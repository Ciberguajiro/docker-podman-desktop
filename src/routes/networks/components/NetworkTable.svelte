<script lang="ts">
  import type { DockerNetwork } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import * as Table from "$lib/components/ui/table";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import NetworkActions from './NetworkActions.svelte';
  import { Copy, ChevronUp, ChevronDown, Network } from "lucide-svelte";

  let {
    networks,
    sortCol,
    sortDesc,
    onSort,
    onCopy,
    onInspect,
    onRemove
  } = $props<{
    networks: DockerNetwork[];
    sortCol: string;
    sortDesc: boolean;
    onSort: (col: string) => void;
    onCopy: (text: string) => void;
    onInspect: (n: DockerNetwork) => void;
    onRemove: (n: DockerNetwork) => void;
  }>();
</script>

<div class="rounded-md border bg-card overflow-hidden">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[25%] cursor-pointer" onclick={() => onSort('name')}>
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
        <Table.Head class="w-[15%] cursor-pointer" onclick={() => onSort('scope')}>
          <div class="flex items-center gap-1">
            {i18n.t('Scope')}
            {#if sortCol === 'scope'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden md:table-cell w-[25%]">{i18n.t('Subnet')}</Table.Head>
        <Table.Head class="text-right w-[20%]">{i18n.t('Actions')}</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each networks as n (n.id)}
        <Table.Row class="group">
          <Table.Cell>
            <div class="flex items-center gap-2">
              <Network class="h-4 w-4 text-muted-foreground" />
              <div class="font-bold truncate">{n.name}</div>
              <Button
                variant="ghost"
                size="icon"
                class="h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={() => onCopy(n.id)}
              >
                <Copy class="h-3 w-3" />
              </Button>
            </div>
          </Table.Cell>
          <Table.Cell>
            <Badge variant="outline">{n.driver}</Badge>
          </Table.Cell>
          <Table.Cell>
            <span class="text-xs text-muted-foreground capitalize">{n.scope}</span>
          </Table.Cell>
          <Table.Cell class="hidden md:table-cell">
            <code class="text-[10px] font-mono">{n.subnet || 'N/A'}</code>
          </Table.Cell>
          <Table.Cell class="text-right">
            <NetworkActions
              network={n}
              {onInspect}
              {onRemove}
            />
          </Table.Cell>
        </Table.Row>
      {/each}
      {#if networks.length === 0}
        <Table.Row>
          <Table.Cell colspan={5} class="h-24 text-center text-muted-foreground">
            {i18n.t('NoNetworksFound')}
          </Table.Cell>
        </Table.Row>
      {/if}
    </Table.Body>
  </Table.Root>
</div>
