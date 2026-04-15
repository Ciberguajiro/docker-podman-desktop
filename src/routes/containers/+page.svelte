<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import type { DockerContainer, DockerImage, ContainerTemplate } from '$lib/types';

  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';
  import ExecModal from '$lib/components/ExecModal.svelte';
  import LogsModal from '$lib/components/LogsModal.svelte';
  import FilesModal from '$lib/components/FilesModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Card from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Accordion from "$lib/components/ui/accordion";
  import { Badge } from "$lib/components/ui/badge";
  import * as Table from "$lib/components/ui/table";
  import * as Tabs from "$lib/components/ui/tabs";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import Container from "$lib/components/ui/Container.svelte";

  import ContainerTable from './components/ContainerTable.svelte';
  import ComposeGroup from './components/ComposeGroup.svelte';
  import TemplateCard from './components/TemplateCard.svelte';

  import {
    RefreshCw,
    Plus,
    Trash2,
    Search,
    Filter,
    Box,
    Package,
    LayoutGrid,
    Rocket
  } from "lucide-svelte";
  import { cn } from '$lib/utils';

  let containers = $state<DockerContainer[]>([]);
  let templates = $state<ContainerTemplate[]>([]);
  let isLoading = $state(true);
  let isTemplatesLoading = $state(false);
  let showAll = $state(false);
  let statusFilter = $state('all');
  let searchQuery = $state('');
  let searchInput = $state('');
  let sortCol = $state('name');
  let sortDesc = $state(false);
  let activeTab = $state('list');

  // Modals state
  let showCreateModal = $state(false);
  let showConfirmRemove = $state(false);
  let containerToRemove = $state<DockerContainer | null>(null);
  let showConfirmPrune = $state(false);
  let showInspectModal = $state(false);
  let inspectData = $state('');
  let inspectTitle = $state('');

  // Exec/Logs/Files state
  let showExecModal = $state(false);
  let execId = $state('');
  let execName = $state('');
  let showLogsModal = $state(false);
  let logsId = $state('');
  let logsName = $state('');
  let showFilesModal = $state(false);
  let filesId = $state('');
  let filesName = $state('');

  // Export state
  let showExportModal = $state(false);
  let exportId = $state('');
  let exportPath = $state('');

  // Unused images
  let unusedImages = $state<DockerImage[]>([]);

  // Create Container form
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
    } catch (e) {
      console.error('Failed to load containers', e);
    } finally {
      isLoading = false;
    }
  }

  async function loadTemplates() {
    isTemplatesLoading = true;
    try {
      templates = await dockerService.getTemplates();
    } catch (e) {
      console.error('Failed to load templates', e);
    } finally {
      isTemplatesLoading = false;
    }
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadContainers();
    }
  });

  onMount(() => {
    loadContainers();
    loadTemplates();
  });

  const filteredContainers = $derived.by(() => {
    let filtered = containers.filter(c =>
      c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      c.image.toLowerCase().includes(searchQuery.toLowerCase())
    );

    if (statusFilter !== 'all') {
      filtered = filtered.filter(c => c.state === statusFilter);
    }

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'image': res = a.image.localeCompare(b.image); break;
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

  async function startContainer(container: DockerContainer) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.startContainer(dockerStore.selectedEngine, container.id);
    if (res.success) {
      toastStore.success('Container started');
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to start container');
    }
  }

  async function stopContainer(container: DockerContainer) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.stopContainer(dockerStore.selectedEngine, container.id);
    if (res.success) {
      toastStore.success('Container stopped');
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to stop container');
    }
  }

  async function removeContainer() {
    if (!dockerStore.selectedEngine || !containerToRemove) return;
    const res = await dockerService.removeContainer(dockerStore.selectedEngine, containerToRemove.id);
    if (res.success) {
      toastStore.success('Container removed');
      showConfirmRemove = false;
      containerToRemove = null;
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to remove container');
    }
  }

  async function createContainer() {
    if (!dockerStore.selectedEngine || !newImage) return;
    const res = await dockerService.createContainer(dockerStore.selectedEngine, {
      image: newImage,
      name: newName,
      ports: newPorts,
      envs: newEnvs,
      volumes: newVolumes,
      restart_policy: newRestartPolicy
    });

    if (res.success) {
      toastStore.success('Container created and started');
      showCreateModal = false;
      // Reset form
      newImage = '';
      newName = '';
      newPorts = '';
      newEnvs = '';
      newVolumes = '';
      newRestartPolicy = 'no';
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to create container');
    }
  }

  function onDeployTemplate(template: ContainerTemplate) {
    newImage = template.image;
    newName = template.id + '-' + Math.random().toString(36).substring(7);
    newPorts = template.ports;
    newEnvs = template.envs;
    newVolumes = template.volumes;
    newRestartPolicy = 'always';
    showCreateModal = true;
  }

  async function pruneContainers() {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.prune(dockerStore.selectedEngine, 'containers');
    if (res.success) {
      toastStore.success('Stopped containers pruned');
      showConfirmPrune = false;
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to prune containers');
    }
  }

  async function inspectContainer(container: DockerContainer) {
    if (!dockerStore.selectedEngine) return;
    try {
      const data = await dockerService.inspect(dockerStore.selectedEngine, container.id);
      inspectData = data;
      inspectTitle = `Inspect: ${container.name}`;
      showInspectModal = true;
    } catch (e) {
      console.error('Failed to inspect container', e);
    }
  }

  async function loadUnusedImages() {
    if (!dockerStore.selectedEngine) return;
    try {
      const allImages = await dockerService.getImages(dockerStore.selectedEngine);
      // Filter images that are not used by any container
      const usedImages = new Set(containers.map(c => c.image));
      unusedImages = allImages.filter(img => {
        const fullRepo = `${img.repository}:${img.tag}`;
        return !usedImages.has(fullRepo) && !usedImages.has(img.id) && !usedImages.has(img.id.replace('sha256:', '').slice(0, 12));
      });
    } catch (e) {
      console.error('Failed to load unused images', e);
    }
  }

  function openExec(container: DockerContainer) {
    execId = container.id;
    execName = container.name;
    showExecModal = true;
  }

  function openLogs(container: DockerContainer) {
    logsId = container.id;
    logsName = container.name;
    showLogsModal = true;
  }

  function openFiles(container: DockerContainer) {
    filesId = container.id;
    filesName = container.name;
    showFilesModal = true;
  }

  function openExport(container: DockerContainer) {
    exportId = container.id;
    exportPath = `container_${container.id.slice(0, 8)}.tar`;
    showExportModal = true;
  }

  async function exportContainer() {
    if (!dockerStore.selectedEngine || !exportId) return;
    const res = await dockerService.exportContainer(dockerStore.selectedEngine, exportId, exportPath);
    if (res.success) {
      toastStore.success(`Container exported to ${exportPath}`);
      showExportModal = false;
    } else {
      toastStore.error(res.error || 'Failed to export container');
    }
  }

  async function onComposeUp(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeUp(dockerStore.selectedEngine, project);
    if (res.success) {
      toastStore.success(`Compose project ${project} up`);
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to compose up');
    }
  }

  async function onComposeRestart(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeRestart(dockerStore.selectedEngine, project);
    if (res.success) {
      toastStore.success(`Compose project ${project} restarted`);
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to compose restart');
    }
  }

  async function onComposeDown(project: string) {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.composeDown(dockerStore.selectedEngine, project);
    if (res.success) {
      toastStore.success(`Compose project ${project} down`);
      loadContainers();
    } else {
      toastStore.error(res.error || 'Failed to compose down');
    }
  }

  function toggleSort(col: string) {
    if (sortCol === col) {
      sortDesc = !sortDesc;
    } else {
      sortCol = col;
      sortDesc = false;
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      toastStore.success(i18n.t('CopiedToClipboard') || 'Copied to clipboard');
    }).catch(() => {
      toastStore.error(i18n.t('FailedToCopy') || 'Failed to copy');
    });
  }
</script>

<Container>
  <PageHeader
    title={i18n.t('Containers')}
    description="Manage your running and stopped containers."
    icon={Box}
  >
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" onclick={loadContainers} disabled={isLoading}>
        <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
        {i18n.t('Refresh')}
      </Button>
      <Button size="sm" onclick={() => {
        // Reset form for new manual container
        newImage = ''; newName = ''; newPorts = ''; newEnvs = ''; newVolumes = ''; newRestartPolicy = 'no';
        showCreateModal = true;
      }}>
        <Plus class="h-4 w-4 mr-2" />
        {i18n.t('NewContainer')}
      </Button>
      <Button variant="destructive" size="sm" onclick={() => showConfirmPrune = true}>
        <Trash2 class="h-4 w-4 mr-2" />
        {i18n.t('Prune')}
      </Button>
    </div>
  </PageHeader>

  <Tabs.Root value={activeTab} onValueChange={v => activeTab = v || 'list'} class="w-full">
    <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-6">
      <Tabs.List class="grid w-full md:w-[400px] grid-cols-2">
        <Tabs.Trigger value="list" class="gap-2">
          <LayoutGrid class="h-4 w-4" />
          {i18n.t('ContainersList')}
        </Tabs.Trigger>
        <Tabs.Trigger value="quickstart" class="gap-2">
          <Rocket class="h-4 w-4" />
          {i18n.t('QuickStart')}
        </Tabs.Trigger>
      </Tabs.List>

      {#if activeTab === 'list'}
      <div class="flex flex-col md:flex-row gap-4 items-center w-full md:w-auto">
        <div class="relative flex-1 w-full md:w-64">
          <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            type="search"
            placeholder={i18n.t('Search')}
            class="pl-9"
            bind:value={searchInput}
          />
        </div>
        <div class="flex items-center gap-2 w-full md:w-auto">
          <div class="flex items-center space-x-2 bg-muted/50 px-3 py-2 rounded-md border h-10">
            <Checkbox id="show-all" bind:checked={showAll} onCheckedChange={loadContainers} />
            <Label for="show-all" class="text-xs font-medium cursor-pointer whitespace-nowrap">{i18n.t('ShowAll')}</Label>
          </div>
          <Select.Root type="single" value={statusFilter} onValueChange={v => statusFilter = v || 'all'}>
            <Select.Trigger class="w-[140px] h-10">
              <Filter class="h-3.5 w-3.5 mr-2" />
              {statusFilter === 'all' ? i18n.t('AllStatuses') : statusFilter.charAt(0).toUpperCase() + statusFilter.slice(1)}
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="all">{i18n.t('AllStatuses')}</Select.Item>
              <Select.Item value="running">{i18n.t('Running')}</Select.Item>
              <Select.Item value="exited">{i18n.t('Stopped')}</Select.Item>
              <Select.Item value="created">{i18n.t('CreatedState')}</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
      </div>
      {/if}
    </div>

    <Tabs.Content value="list" class="space-y-6 mt-0">
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
      <div class="mt-8 border rounded-lg overflow-hidden bg-card">
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
    </Tabs.Content>

    <Tabs.Content value="quickstart" class="mt-0">
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3 mt-6">
        {#if isTemplatesLoading}
          {#each Array(3) as i_}
            <Card.Root class="h-48 animate-pulse bg-muted/20" />
          {/each}
        {:else}
          {#each templates as template (template.id)}
            <TemplateCard {template} onDeploy={onDeployTemplate} />
          {/each}

          {#if templates.length === 0}
             <div class="col-span-full flex flex-col items-center justify-center py-20 text-center">
              <div class="bg-muted p-4 rounded-full mb-4">
                <Rocket class="h-10 w-10 text-muted-foreground/50" />
              </div>
              <h3 class="text-lg font-medium">No templates found</h3>
              <p class="text-sm text-muted-foreground max-w-xs mt-1">
                Add JSON templates to your config directory to see them here.
              </p>
            </div>
          {/if}
        {/if}
      </div>
    </Tabs.Content>
  </Tabs.Root>
</Container>

<!-- Modals -->
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

{#if showInspectModal}
<InspectModal
  title={inspectTitle}
  data={inspectData}
  bind:show={showInspectModal}
/>
{/if}

{#if showExecModal}
<ExecModal
  containerId={execId}
  containerName={execName}
  bind:show={showExecModal}
/>
{/if}

{#if showLogsModal}
<LogsModal
  containerId={logsId}
  containerName={logsName}
  bind:show={showLogsModal}
/>
{/if}

{#if showFilesModal}
<FilesModal
  containerId={filesId}
  containerName={filesName}
  bind:show={showFilesModal}
/>
{/if}

<style>
  :global(.tabs-content) {
    outline: none !important;
  }
</style>
