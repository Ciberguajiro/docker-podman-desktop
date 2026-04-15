<script lang="ts">
  import type { DockerContainer } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import ContainerTable from './ContainerTable.svelte';
  import { Boxes, Play, RotateCw, Square } from "lucide-svelte";

  let {
    project,
    containers,
    onComposeUp,
    onComposeRestart,
    onComposeDown,
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
    project: string;
    containers: DockerContainer[];
    onComposeUp: (project: string) => void;
    onComposeRestart: (project: string) => void;
    onComposeDown: (project: string) => void;
    sortCol: string;
    sortDesc: boolean;
    onSort: (col: string) => void;
    onCopy: (text: string) => void;
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

<Card.Root class="mb-6 overflow-hidden border-sidebar-border/50 shadow-sm">
  <Card.Header class="py-3 px-4 bg-muted/50 flex flex-row items-center justify-between space-y-0">
    <div class="flex items-center gap-2">
      <Boxes class="h-4 w-4 text-primary" />
      <Card.Title class="text-xs uppercase tracking-widest text-muted-foreground font-bold">
        {i18n.t('Project')}: {project}
      </Card.Title>
    </div>
    <div class="flex items-center gap-1">
      <Button variant="ghost" size="sm" class="h-8 text-[10px] gap-1.5" onclick={() => onComposeUp(project)}>
        <Play class="h-3 w-3" /> {i18n.t('Up')}
      </Button>
      <Button variant="ghost" size="sm" class="h-8 text-[10px] gap-1.5" onclick={() => onComposeRestart(project)}>
        <RotateCw class="h-3 w-3" /> {i18n.t('Restart')}
      </Button>
      <Button variant="ghost" size="sm" class="h-8 text-[10px] gap-1.5 text-destructive hover:text-destructive" onclick={() => onComposeDown(project)}>
        <Square class="h-3 w-3" /> {i18n.t('Down')}
      </Button>
    </div>
  </Card.Header>
  <Card.Content class="p-0">
    <ContainerTable
      {containers}
      {sortCol}
      {sortDesc}
      {onSort}
      {onCopy}
      {onStart}
      {onStop}
      {onExec}
      {onFiles}
      {onLogs}
      {onExport}
      {onInspect}
      {onRemove}
    />
  </Card.Content>
</Card.Root>
