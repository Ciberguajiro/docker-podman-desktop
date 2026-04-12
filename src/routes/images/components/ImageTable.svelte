<script lang="ts">
  import type { DockerImage } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import * as Table from "$lib/components/ui/table";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import ImageActions from './ImageActions.svelte';
  import { Copy, ChevronUp, ChevronDown } from "lucide-svelte";

  let {
    images,
    sortCol,
    sortDesc,
    onSort,
    onCopy,
    onRun,
    onTag,
    onInspect,
    onHistory,
    onRemove
  } = $props<{
    images: DockerImage[];
    sortCol: string;
    sortDesc: boolean;
    onSort: (col: string) => void;
    onCopy: (text: string) => void;
    onRun: (img: DockerImage) => void;
    onTag: (img: DockerImage) => void;
    onInspect: (img: DockerImage) => void;
    onHistory: (img: DockerImage) => void;
    onRemove: (img: DockerImage) => void;
  }>();
</script>

<div class="rounded-md border bg-card overflow-hidden">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[30%] cursor-pointer" onclick={() => onSort('repository')}>
          <div class="flex items-center gap-1">
            {i18n.t('Repository')}
            {#if sortCol === 'repository'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden md:table-cell w-[15%] cursor-pointer" onclick={() => onSort('tag')}>
          <div class="flex items-center gap-1">
            {i18n.t('Tag')}
            {#if sortCol === 'tag'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden lg:table-cell w-[15%] cursor-pointer" onclick={() => onSort('id')}>
          <div class="flex items-center gap-1">
            {i18n.t('ImageID')}
            {#if sortCol === 'id'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="w-[10%] cursor-pointer" onclick={() => onSort('size')}>
          <div class="flex items-center gap-1">
            {i18n.t('Size')}
            {#if sortCol === 'size'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="hidden sm:table-cell w-[15%] cursor-pointer" onclick={() => onSort('created')}>
          <div class="flex items-center gap-1">
            {i18n.t('Created')}
            {#if sortCol === 'created'}
              {#if sortDesc}<ChevronDown class="h-3 w-3" />{:else}<ChevronUp class="h-3 w-3" />{/if}
            {/if}
          </div>
        </Table.Head>
        <Table.Head class="text-right w-[15%]">{i18n.t('Actions')}</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each images as img (img.id)}
        <Table.Row class="group">
          <Table.Cell>
            <div class="font-bold text-primary truncate">{img.repository}</div>
          </Table.Cell>
          <Table.Cell class="hidden md:table-cell">
            <Badge variant="outline" class="font-medium">{img.tag}</Badge>
          </Table.Cell>
          <Table.Cell class="hidden lg:table-cell">
            <div class="flex items-center gap-1.5 group/id">
              <code class="text-[10px] text-muted-foreground font-mono">{img.id.slice(0, 12)}</code>
              <Button
                variant="ghost"
                size="icon"
                class="h-4 w-4 opacity-0 group-hover/id:opacity-100 transition-opacity"
                onclick={() => onCopy(img.id)}
              >
                <Copy class="h-3 w-3" />
              </Button>
            </div>
          </Table.Cell>
          <Table.Cell class="text-xs">{img.size}</Table.Cell>
          <Table.Cell class="hidden sm:table-cell text-xs text-muted-foreground">{img.created}</Table.Cell>
          <Table.Cell class="text-right">
            <ImageActions
              image={img}
              {onRun}
              {onTag}
              {onInspect}
              {onHistory}
              {onRemove}
            />
          </Table.Cell>
        </Table.Row>
      {/each}
      {#if images.length === 0}
        <Table.Row>
          <Table.Cell colspan={6} class="h-24 text-center text-muted-foreground">
            {i18n.t('NoImagesFound')}
          </Table.Cell>
        </Table.Row>
      {/if}
    </Table.Body>
  </Table.Root>
</div>
