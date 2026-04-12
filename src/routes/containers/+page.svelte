<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import { cn, sanitize, sanitizePorts } from '$lib/utils';
  import type { DockerContainer, DockerImage, CommandResult } from '$lib/types';

  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';
  import ExecModal from '$lib/components/ExecModal.svelte';
  import LogsModal from '$lib/components/LogsModal.svelte';
  import FilesModal from '$lib/components/FilesModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import * as Select from "$lib/components/ui/select";
  import * as Tabs from "$lib/components/ui/tabs";
  import * as Card from "$lib/components/ui/card";
  import * as Accordion from "$lib/components/ui/accordion";
  import * as Table from "$lib/components/ui/table";
  import { Separator } from "$lib/components/ui/separator";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";

  import ContainerTable from './components/ContainerTable.svelte';
  import ComposeGroup from './components/ComposeGroup.svelte';
  import {
    RefreshCw,
    Plus,
    Trash2,
    Search,
    Filter,
    Package,
    Activity,
    Box
  } from "lucide-svelte";

  let containers = $state<DockerContainer[]>([]);
  let isLoading = $state(true);
  let lastRefreshed = $state<Date | null>(null);
  let searchQuery = $state('');
  let searchInput = $state('');
  let statusFilter = $state('all');
  let images = $state<DockerImage[]>([]);
  let showAll = $state(true);
  let sortCol = $state('name');
  let sortDesc = $state(false);

  // Modals state
  let showCreateModal = $state(false);
  let showConfirmRemove = $state(false);
  let containerToRemove = $state<DockerContainer | null>(null);
  let showConfirmPrune = $state(false);
  let showInspectModal = $state(false);
  let inspectTitle = $state('');
  let inspectData = $state('');
  let showExecModal = $state(false);
  let execId = $state('');
  let execName = $state('');
  let showLogsModal = $state(false);
  let logsId = $state('');
  let logsName = $state('');
  let showFilesModal = $state(false);
  let filesId = $state('');
  let filesName = $state('');
  let showExportModal = $state(false);
  let exportPath = $state('');
  let containerToExport = $state<DockerContainer | null>(null);

  // Create container state
  let newImage = $state('');
  let newName = $state('');
  let newPorts = $state('');
  let newEnvs = $state('');
  let newVolumes = $state('');
  let newRestartPolicy = $state('no');

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  async function loadContainers() {
    if (!dockerStore.selectedEngine) return;
    isLoading = true;
    try {
      containers = await dockerService.getContainers(dockerStore.selectedEngine, showAll);
      lastRefreshed = new Date();
    } catch (e) {
      console.error('Failed to load containers', e);
      toastStore.error(`Failed to load containers: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadContainers();
    }
  });

  onMount(() => {
    loadContainers();
  });

  async function loadUnusedImages() {
    if (!dockerStore.selectedEngine) return;
    try {
      images = await dockerService.getImages(dockerStore.selectedEngine);
    } catch (e) {
      console.error('Failed to load images', e);
    }
  }

  const filteredContainers = $derived.by(() => {
    let filtered = containers.filter(c => {
      const matchesQuery = !searchQuery ||
        c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        c.image.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (c.compose_project && c.compose_project.toLowerCase().includes(searchQuery.toLowerCase()));
      const matchesStatus = statusFilter === 'all' || c.state.toLowerCase() === statusFilter;
      return matchesQuery && matchesStatus;
    });

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'image': res = a.image.localeCompare(b.image); break;
        case 'status': res = a.status.localeCompare(b.status); break;
        case 'state': res = a.state.localeCompare(b.state); break;
        default: res = a.name.localeCompare(b.name);
      }
      return sortDesc ? -res : res;
    });
  });

  const groupedContainers = $derived.by(() => {
    const groups: Record<string, DockerContainer[]> = {};
    const standalone: DockerContainer[] = [];

    filteredContainers.forEach(c => {
      if (c.compose_project) {
        if (!groups[c.compose_project]) groups[c.compose_project] = [];
        groups[c.compose_project].push(c);
      } else {
        standalone.push(c);
      }
    });

    return { groups, standalone };
  });

  const unusedImages = $derived.by(() => {
    return images.filter(img => {
      const imgFullName = `${img.repository}:${img.tag}`;
      return !containers.some(c => c.image === img.repository || c.image === imgFullName || c.image === img.id);
    });
  });

  async function startContainer(id: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.startContainer(dockerStore.selectedEngine, id);
    if (res.success) toastStore.success(`Container ${id} started`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function stopContainer(id: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.stopContainer(dockerStore.selectedEngine, id);
    if (res.success) toastStore.success(`Container ${id} stopped`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function removeContainer() {
    if (!containerToRemove || !dockerStore.selectedEngine) return;
    const res = await dockerService.removeContainer(dockerStore.selectedEngine, containerToRemove.id);
    if (res.success) toastStore.success(`Container ${containerToRemove.name} removed`);
    else toastStore.error(`Error: ${res.error}`);
    showConfirmRemove = false;
    loadContainers();
  }

  async function pruneContainers() {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.prune(dockerStore.selectedEngine, 'containers');
    if (res.success) toastStore.success('Stopped containers pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    showConfirmPrune = false;
    loadContainers();
  }

  async function createContainer() {
    if (!newImage || !dockerStore.selectedEngine) return;
    const res = await dockerService.createContainer(dockerStore.selectedEngine, {
      image: sanitize(newImage),
      name: sanitize(newName),
      ports: sanitizePorts(newPorts),
      envs: sanitize(newEnvs),
      volumes: sanitize(newVolumes),
      restartPolicy: newRestartPolicy
    });
    if (res.success) {
      toastStore.success(`Container created from ${newImage}`);
      showCreateModal = false;
      newImage = ''; newName = ''; newPorts = ''; newEnvs = ''; newVolumes = ''; newRestartPolicy = 'no';
    } else {
      toastStore.error(`Error: ${res.error}`);
    }
    loadContainers();
  }

  async function inspectContainer(c: DockerContainer) {
    if (!dockerStore.selectedEngine) return;
    try {
      inspectTitle = `Inspect Container: ${c.name}`;
      inspectData = await dockerService.inspect(dockerStore.selectedEngine, c.id);
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect container: ${e}`);
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toastStore.success(i18n.t('CopiedToClipboard'));
    } catch (err) {
      toastStore.error(i18n.t('FailedToCopy'));
    }
  }

  async function exportContainer() {
    if (!containerToExport || !exportPath || !dockerStore.selectedEngine) return;
    const res = await dockerService.exportContainer(dockerStore.selectedEngine, containerToExport.id, exportPath);
    if (res.success) {
      toastStore.success(`Container exported to ${exportPath}`);
      showExportModal = false;
    } else {
      toastStore.error(`Export failed: ${res.error}`);
    }
  }

  async function composeUp(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeUp(dockerStore.selectedEngine, project);
    if (res.success) toastStore.success(`Compose up: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function composeDown(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeDown(dockerStore.selectedEngine, project);
    if (res.success) toastStore.success(`Compose down: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function composeRestart(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeRestart(dockerStore.selectedEngine, project);
    if (res.success) toastStore.success(`Compose restart: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  function toggleSort(col: string) {
    if (sortCol === col) sortDesc = !sortDesc;
    else {
      sortCol = col;
      sortDesc = false;
    }
  }

  function openExec(c: DockerContainer) {
    execId = c.id;
    execName = c.name;
    showExecModal = true;
  }

  function openLogs(c: DockerContainer) {
    logsId = c.id;
    logsName = c.name;
    showLogsModal = true;
  }

  function openFiles(c: DockerContainer) {
    filesId = c.id;
    filesName = c.name;
    showFilesModal = true;
  }

  function openExport(c: DockerContainer) {
    containerToExport = c;
    exportPath = `${c.name}.tar`;
    showExportModal = true;
  }
</script>

<div class="h-full flex flex-col bg-background">
  <!-- Header -->
  <header class="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-10">
    <div class="container flex flex-col md:flex-row md:items-center justify-between py-4 gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-primary/10 rounded-lg">
          <Package class="h-6 w-6 text-primary" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{i18n.t('Containers')}</h1>
          <p class="text-xs text-muted-foreground flex items-center gap-2">
            <Activity class="h-3 w-3" />
            {containers.length} {i18n.t('Containers').toLowerCase()}
            {#if lastRefreshed}
              • {i18n.t('LastUpdated')}: {lastRefreshed.toLocaleTimeString()}
            {/if}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" onclick={loadContainers} disabled={isLoading}>
          <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
          {i18n.t('Refresh')}
        </Button>
        <Button size="sm" onclick={() => showCreateModal = true}>
          <Plus class="h-4 w-4 mr-2" />
          {i18n.t('Container')}
        </Button>
        <Button variant="destructive" size="sm" onclick={() => showConfirmPrune = true}>
          <Trash2 class="h-4 w-4 mr-2" />
          {i18n.t('Prune')}
        </Button>
      </div>
    </div>

    <!-- Filters -->
    <div class="container pb-4">
      <div class="flex flex-col md:flex-row gap-4">
        <div class="relative flex-1">
          <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            type="search"
            placeholder={i18n.t('Search')}
            class="pl-9"
            bind:value={searchInput}
          />
        </div>
        <div class="flex items-center gap-2">
          <div class="flex items-center space-x-2 bg-muted/50 px-3 py-1.5 rounded-md border">
            <Checkbox id="show-all" bind:checked={showAll} onCheckedChange={loadContainers} />
            <Label for="show-all" class="text-xs font-medium cursor-pointer">{i18n.t('ShowAll')}</Label>
          </div>
          <Select.Root type="single" value={statusFilter} onValueChange={v => statusFilter = v || 'all'}>
            <Select.Trigger class="w-[140px] h-9">
              <Filter class="h-3.5 w-3.5 mr-2" />
              {statusFilter === 'all' ? i18n.t('AllStatuses') : statusFilter.charAt(0).toUpperCase() + statusFilter.slice(1)}
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="all">{i18n.t('AllStatuses')}</Select.Item>
              <Select.Item value="running">{i18n.t('Running')}</Select.Item>
              <Select.Item value="exited">{i18n.t('Exited')}</Select.Item>
              <Select.Item value="created">{i18n.t('Created')}</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
      </div>
    </div>
  </header>

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    <div class="container py-6">
      {#if isLoading && containers.length === 0}
        <div class="grid gap-4">
          {#each Array(3) as i_}
            <Card.Root class="h-32 animate-pulse bg-muted/20" />
          {/each}
        </div>
      {:else}
        <!-- Grouped by Project (Compose) -->
        {#each Object.entries(groupedContainers.groups) as [project, groupContainers] (project)}
          <ComposeGroup
            {project}
            containers={groupContainers}
            {onComposeUp}
            {onComposeRestart}
            {onComposeDown}
            sortCol={sortCol}
            sortDesc={sortDesc}
            onSort={toggleSort}
            onCopy={copyToClipboard}
            onStart={startContainer}
            onStop={stopContainer}
            onExec={openExec}
            onFiles={openFiles}
            onLogs={openLogs}
            onExport={openExport}
            onInspect={inspectContainer}
            onRemove={(c) => { containerToRemove = c; showConfirmRemove = true; }}
          />
        {/each}

        <!-- Standalone Containers -->
        {#if groupedContainers.standalone.length > 0}
          <div class="space-y-4">
            <div class="flex items-center gap-2 px-1">
              <Box class="h-4 w-4 text-muted-foreground" />
              <h2 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                {i18n.t('StandaloneContainers')}
              </h2>
            </div>
            <ContainerTable
              containers={groupedContainers.standalone}
              {sortCol}
              {sortDesc}
              onSort={toggleSort}
              onCopy={copyToClipboard}
              onStart={startContainer}
              onStop={stopContainer}
              onExec={openExec}
              onFiles={openFiles}
              onLogs={openLogs}
              onExport={openExport}
              onInspect={inspectContainer}
              onRemove={(c) => { containerToRemove = c; showConfirmRemove = true; }}
            />
          </div>
        {/if}

        {#if containers.length === 0 && !isLoading}
          <div class="flex flex-col items-center justify-center py-20 text-center">
            <div class="bg-muted p-4 rounded-full mb-4">
              <Package class="h-10 w-10 text-muted-foreground/50" />
            </div>
            <h3 class="text-lg font-medium">{i18n.t('NoContainersFound')}</h3>
            <p class="text-sm text-muted-foreground max-w-xs mt-1">
              Try changing your filters or create a new container to get started.
            </p>
          </div>
        {/if}
      {/if}

      <!-- Unused Images Collapsible -->
      <div class="mt-12 border rounded-lg overflow-hidden bg-card">
        <Accordion.Root type="single" onValueChange={v => v === 'unused-images' && loadUnusedImages()}>
          <Accordion.Item value="unused-images" class="border-none">
            <Accordion.Trigger class="px-4 py-3 hover:bg-muted/50 transition-colors">
              <span class="flex items-center gap-2 font-medium">
                <Trash2 class="h-4 w-4 text-muted-foreground" />
                {i18n.t('UnusedImages')}
                <Badge variant="secondary" class="ml-2">{unusedImages.length}</Badge>
              </span>
            </Accordion.Trigger>
            <Accordion.Content class="p-0 border-t">
              <Table.Root>
                <Table.Header>
                  <Table.Row class="bg-muted/30">
                    <Table.Head>{i18n.t('ImageID')}</Table.Head>
                    <Table.Head>{i18n.t('Repository')}</Table.Head>
                    <Table.Head>{i18n.t('Tag')}</Table.Head>
                    <Table.Head class="text-right">{i18n.t('Size')}</Table.Head>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {#each unusedImages as img (img.id)}
                    <Table.Row>
                      <Table.Cell class="font-mono text-[10px] text-muted-foreground">{img.id.slice(0, 12)}</Table.Cell>
                      <Table.Cell class="font-medium text-xs">{img.repository}</Table.Cell>
                      <Table.Cell><Badge variant="outline" class="text-[10px] px-1 h-4">{img.tag}</Badge></Table.Cell>
                      <Table.Cell class="text-right text-xs text-muted-foreground">{img.size}</Table.Cell>
                    </Table.Row>
                  {/each}
                  {#if unusedImages.length === 0}
                    <Table.Row>
                      <Table.Cell colspan={4} class="h-16 text-center text-muted-foreground text-sm italic">
                        No unused images found.
                      </Table.Cell>
                    </Table.Row>
                  {/if}
                </Table.Body>
              </Table.Root>
            </Accordion.Content>
          </Accordion.Item>
        </Accordion.Root>
      </div>
    </div>
  </div>
</div>

<!-- Modals -->
<!-- Create Container Modal -->
{#if showCreateModal}
<ConfirmationModal
  title={i18n.t('NewContainer')}
  message=""
  onConfirm={createContainer}
  onCancel={() => showCreateModal = false}
>
  <div class="grid gap-4 py-4">
    <div class="grid gap-2">
      <Label for="image">{i18n.t('Image')}*</Label>
      <Input id="image" bind:value={newImage} placeholder="e.g. nginx:latest" />
    </div>
    <div class="grid gap-2">
      <Label for="name">{i18n.t('Name')}</Label>
      <Input id="name" bind:value={newName} placeholder="e.g. my-web-server" />
    </div>
    <div class="grid gap-2">
      <Label for="ports">{i18n.t('Ports')}</Label>
      <Input id="ports" bind:value={newPorts} placeholder="e.g. 80:80, 443:443" />
    </div>
    <div class="grid gap-2">
      <Label for="envs">{i18n.t('EnvironmentVariables')}</Label>
      <Input id="envs" bind:value={newEnvs} placeholder="e.g. KEY=VAL, DEBUG=true" />
    </div>
    <div class="grid gap-2">
      <Label for="volumes">{i18n.t('Volumes')}</Label>
      <Input id="volumes" bind:value={newVolumes} placeholder="e.g. /host/path:/container/path" />
    </div>
    <div class="grid gap-2">
      <Label for="restart">{i18n.t('RestartPolicy')}</Label>
      <Select.Root type="single" value={newRestartPolicy} onValueChange={v => newRestartPolicy = v || 'no'}>
        <Select.Trigger id="restart">
          {newRestartPolicy}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="no">no</Select.Item>
          <Select.Item value="always">always</Select.Item>
          <Select.Item value="on-failure">on-failure</Select.Item>
          <Select.Item value="unless-stopped">unless-stopped</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>
  </div>
</ConfirmationModal>
{/if}

<!-- Remove Container Confirm -->
{#if showConfirmRemove && containerToRemove}
<ConfirmationModal
  title={i18n.t('ConfirmRemove')}
  message={`${i18n.t('ConfirmRemoveMessage')} "${containerToRemove.name}"?`}
  onConfirm={removeContainer}
  onCancel={() => showConfirmRemove = false}
  confirmText={i18n.t('Remove')}
  isDestructive={true}
/>
{/if}

<!-- Prune Confirm -->
{#if showConfirmPrune}
<ConfirmationModal
  title={i18n.t('ConfirmPrune')}
  message={i18n.t('ConfirmPruneMessage')}
  onConfirm={pruneContainers}
  onCancel={() => showConfirmPrune = false}
  confirmText={i18n.t('Prune')}
  isDestructive={true}
/>
{/if}

<!-- Export Container Modal -->
{#if showExportModal}
<ConfirmationModal
  title={i18n.t('ExportContainer')}
  message=""
  onConfirm={exportContainer}
  onCancel={() => showExportModal = false}
>
  <div class="grid gap-2 py-4">
    <Label for="export-path">{i18n.t('ExportPath')}</Label>
    <Input id="export-path" bind:value={exportPath} placeholder="container_export.tar" />
  </div>
</ConfirmationModal>
{/if}

<!-- Other Modals -->
{#if showInspectModal}
<InspectModal
  title={inspectTitle}
  data={inspectData}
  onClose={() => showInspectModal = false}
/>
{/if}

{#if showExecModal}
<ExecModal
  containerId={execId}
  containerName={execName}
  onClose={() => showExecModal = false}
/>
{/if}

{#if showLogsModal}
<LogsModal
  containerId={logsId}
  containerName={logsName}
  onClose={() => showLogsModal = false}
/>
{/if}

{#if showFilesModal}
<FilesModal
  containerId={filesId}
  containerName={filesName}
  onClose={() => showFilesModal = false}
/>
{/if}
