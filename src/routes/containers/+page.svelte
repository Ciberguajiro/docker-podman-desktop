<script lang="ts">
  import { onMount } from 'svelte';
  import { flip } from 'svelte/animate';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { sanitize, sanitizePorts } from '$lib/utils';
  import type { DockerContainer, DockerImage, CommandResult } from '$lib/types';
  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';
  import ExecModal from '$lib/components/ExecModal.svelte';
  import LogsModal from '$lib/components/LogsModal.svelte';
  import FilesModal from '$lib/components/FilesModal.svelte';
  import VirtualList from '$lib/components/VirtualList.svelte';

  let containers = $state<DockerContainer[]>([]);
  let isLoading = $state(true);
  let lastRefreshed = $state<Date | null>(null);
  let searchQuery = $state('');
  let searchInput = $state('');
  let statusFilter = $state('all');

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });
  let images = $state<DockerImage[]>([]);
  let showAll = $state(true);

  let sortCol = $state('name');
  let sortDesc = $state(false);

  let showCreateModal = $state(false);
  let newImage = $state('');
  let newName = $state('');
  let newPorts = $state('');
  let newEnvs = $state('');
  let newVolumes = $state('');
  let newRestartPolicy = $state('no');

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

  async function loadContainers() {
    isLoading = true;
    try {
      containers = await dockerStore.invoke<DockerContainer[]>('docker_ps', { all: showAll });
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
    try {
      images = await dockerStore.invoke<DockerImage[]>('docker_images');
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

  const unusedImages = $derived.by(() => {
    return images.filter(img => {
      const imgFullName = `${img.repository}:${img.tag}`;
      return !containers.some(c => c.image === img.repository || c.image === imgFullName || c.image === img.id);
    });
  });

  async function startContainer(id: string) {
    const res = await dockerStore.invoke<CommandResult>('docker_start', { containerId: id });
    if (res.success) toastStore.success(`Container ${id} started`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function pruneContainers() {
    const res = await dockerStore.invoke<CommandResult>('docker_prune', { type_: 'containers' });
    if (res.success) toastStore.success('Stopped containers pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    loadContainers();
  }

  async function stopContainer(id: string) {
    const res = await dockerStore.invoke<CommandResult>('docker_stop', { containerId: id });
    if (res.success) toastStore.success(`Container ${id} stopped`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function removeContainer() {
    if (!containerToRemove) return;
    const res = await dockerStore.invoke<CommandResult>('docker_remove_container', { containerId: containerToRemove.id, force: true });
    if (res.success) toastStore.success(`Container ${containerToRemove.name} removed`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function createContainer() {
    if (!newImage) return;
    const sanitizedImage = sanitize(newImage);
    const sanitizedName = sanitize(newName);
    const sanitizedPorts = sanitizePorts(newPorts);
    const sanitizedEnvs = sanitize(newEnvs);
    const sanitizedVolumes = sanitize(newVolumes);

    const res = await dockerStore.invoke<CommandResult>('docker_create_container', {
      image: sanitizedImage,
      name: sanitizedName,
      ports: sanitizedPorts,
      envs: sanitizedEnvs,
      volumes: sanitizedVolumes,
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
    try {
      inspectTitle = `Inspect Container: ${c.name}`;
      inspectData = await dockerStore.invoke<string>('docker_inspect', { id: c.id });
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect container: ${e}`);
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toastStore.success(i18n.t('CopiedToClipboard') || 'Copied to clipboard');
    } catch (err) {
      toastStore.error(i18n.t('FailedToCopy') || 'Failed to copy to clipboard');
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

  async function exportContainer() {
    if (!containerToExport || !exportPath) return;
    const res = await dockerStore.invoke<CommandResult>('docker_export_container', {
      containerId: containerToExport.id,
      path: exportPath
    });
    if (res.success) {
      toastStore.success(`Container exported to ${exportPath}`);
      showExportModal = false;
    } else {
      toastStore.error(`Export failed: ${res.error}`);
    }
  }

  async function composeUp(project: string) {
    const res = await dockerStore.invoke<CommandResult>('docker_compose_up', { projectName: project });
    if (res.success) toastStore.success(`Compose up: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function composeDown(project: string) {
    const res = await dockerStore.invoke<CommandResult>('docker_compose_down', { projectName: project });
    if (res.success) toastStore.success(`Compose down: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  async function composeRestart(project: string) {
    const res = await dockerStore.invoke<CommandResult>('docker_compose_restart', { projectName: project });
    if (res.success) toastStore.success(`Compose restart: ${project}`);
    else toastStore.error(`Error: ${res.error}`);
    loadContainers();
  }

  const groupedContainers = $derived.by(() => {
    const groups: Record<string, DockerContainer[]> = {};
    const standalone: DockerContainer[] = [];

    filteredContainers.forEach(c => {
      if (c.compose_project) {
        const project = c.compose_project.split(',')[0];
        if (!groups[project]) {
          groups[project] = [];
        }
        groups[project].push(c);
      } else {
        standalone.push(c);
      }
    });

    return { groups, standalone };
  });

  $effect(() => {
    // Reset when filters change
    searchQuery;
    statusFilter;
  });
</script>

<div class="p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-6 shrink-0">
    <div class="flex items-center gap-4">
      <h1 class="text-3xl font-bold">{i18n.t('Containers')}</h1>
      <div class="relative">
        <input
          type="text"
          placeholder={i18n.t('Search')}
          class="input input-bordered input-sm w-64"
          bind:value={searchInput}
        />
        <span class="absolute right-2 top-1.5 opacity-30">🔍</span>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <select class="select select-bordered select-sm" bind:value={statusFilter}>
        <option value="all">{i18n.t('AllStatus')}</option>
        <option value="running">{i18n.t('Running')}</option>
        <option value="exited">{i18n.t('Stopped')}</option>
        <option value="paused">{i18n.t('Paused')}</option>
        <option value="created">{i18n.t('CreatedState')}</option>
      </select>
      <button class="btn btn-primary btn-sm" onclick={() => { showCreateModal = true; loadUnusedImages(); }}>➕ {i18n.t('Create')}</button>
      <button class="btn btn-error btn-outline btn-sm" onclick={() => (showConfirmPrune = true)}>🧹 {i18n.t('Prune')}</button>
      <div class="form-control">
        <label class="label cursor-pointer gap-2">
          <span class="label-text">{i18n.t('ShowAll')}</span>
          <input type="checkbox" class="checkbox checkbox-primary checkbox-sm" bind:checked={showAll} onchange={loadContainers} />
        </label>
      </div>
      <div class="flex flex-col items-end">
        <button class="btn btn-outline btn-sm" onclick={loadContainers} disabled={isLoading}>
          {#if isLoading}
            <span class="loading loading-spinner loading-xs"></span>
          {/if}
          🔄 {i18n.t('Refresh')}
        </button>
        {#if lastRefreshed}
          <span class="text-[10px] opacity-50 mt-1">{i18n.t('LastUpdated')}: {lastRefreshed.toLocaleTimeString()}</span>
        {/if}
      </div>
    </div>
  </div>

  <ConfirmationModal
    bind:show={showConfirmRemove}
    title={i18n.t('Remove')}
    message={containerToRemove ? i18n.t('ForceRemoveMessage') : ''}
    onConfirm={removeContainer}
  />

  <ConfirmationModal
    bind:show={showConfirmPrune}
    title={i18n.t('Prune')}
    message={i18n.t('AreYouSurePruneContainers') || 'Are you sure you want to remove all stopped containers?'}
    onConfirm={pruneContainers}
  />

  <InspectModal
    bind:show={showInspectModal}
    title={inspectTitle}
    data={inspectData}
  />

  <ExecModal
    bind:show={showExecModal}
    containerId={execId}
    containerName={execName}
  />

  <LogsModal
    bind:show={showLogsModal}
    containerId={logsId}
    containerName={logsName}
  />

  <FilesModal
    bind:show={showFilesModal}
    containerId={filesId}
    containerName={filesName}
  />

  {#if showExportModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('Export')} Container: {containerToExport?.name}</h3>
        <div class="py-4">
          <div class="form-control w-full">
            <label class="label" for="export-path"><span class="label-text">{i18n.t('OutputPath')} (.tar)</span></label>
            <input id="export-path" type="text" class="input input-bordered w-full" bind:value={exportPath} />
            <span class="label-text-alt mt-1 px-1">{i18n.t('FileSavedHost')}</span>
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showExportModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={exportContainer}>{i18n.t('Export')}</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showCreateModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('Create')} Container</h3>
        <div class="space-y-4 py-4">
          <div class="form-control w-full">
            <label class="label" for="new-image"><span class="label-text">{i18n.t('ImageName')} *</span></label>
            <input id="new-image" type="text" placeholder="e.g. nginx:latest" class="input input-bordered w-full" bind:value={newImage} list="image-suggestions" />
            <datalist id="image-suggestions">
              {#each images as img}
                <option value="{img.repository}:{img.tag}"></option>
              {/each}
            </datalist>
          </div>
          <div class="form-control w-full">
            <label class="label" for="new-name"><span class="label-text">{i18n.t('ContainerName')}</span></label>
            <input id="new-name" type="text" placeholder="e.g. my-web-app" class="input input-bordered w-full" bind:value={newName} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="new-ports"><span class="label-text">{i18n.t('Ports')} (host:container)</span></label>
            <input id="new-ports" type="text" placeholder="e.g. 8080:80" class="input input-bordered w-full" bind:value={newPorts} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="new-envs"><span class="label-text">{i18n.t('EnvVars')} (KEY=VAL, ...)</span></label>
            <input id="new-envs" type="text" placeholder="e.g. NODE_ENV=production,PORT=3000" class="input input-bordered w-full" bind:value={newEnvs} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="new-restart-policy"><span class="label-text">{i18n.t('RestartPolicy')}</span></label>
            <select id="new-restart-policy" class="select select-bordered w-full" bind:value={newRestartPolicy}>
              <option value="no">no</option>
              <option value="always">always</option>
              <option value="on-failure">on-failure</option>
              <option value="unless-stopped">unless-stopped</option>
            </select>
          </div>
          <div class="form-control w-full">
            <label class="label" for="new-volumes"><span class="label-text">{i18n.t('VolumesMapping')} (host:container, ...)</span></label>
            <input id="new-volumes" type="text" placeholder="e.g. /host/path:/container/path, myvol:/data" class="input input-bordered w-full" bind:value={newVolumes} />
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showCreateModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={createContainer}>{i18n.t('Create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="space-y-4 flex-1 overflow-auto">
    {#if isLoading && containers.length === 0}
      <div class="card bg-base-200 shadow-xl p-4 space-y-4">
        <div class="skeleton h-8 w-full"></div>
        <div class="skeleton h-12 w-full"></div>
      </div>
    {:else}
      {#each Object.entries(groupedContainers.groups) as [project, groupContainers] (project)}
        <div class="card bg-base-300 shadow-sm border border-base-content/10 mb-4 shrink-0">
          <div class="card-body p-4">
            <div class="flex justify-between items-center mb-2">
              <div class="flex items-center gap-2">
                <span class="text-xl">📦</span>
                <h2 class="card-title text-sm uppercase tracking-widest opacity-70">{i18n.t('Project')}: {project}</h2>
              </div>
              <div class="join">
                <button class="btn btn-ghost btn-xs join-item" onclick={() => composeUp(project)} title={i18n.t('Up')}>▶️ {i18n.t('Up')}</button>
                <button class="btn btn-ghost btn-xs join-item" onclick={() => composeRestart(project)} title={i18n.t('Restart')}>🔄 {i18n.t('Restart')}</button>
                <button class="btn btn-ghost btn-xs join-item text-error" onclick={() => composeDown(project)} title={i18n.t('Down')}>⏹️ {i18n.t('Down')}</button>
              </div>
            </div>
            <div class="overflow-x-auto">
              <table class="table table-sm w-full">
                <thead>
                  <tr class="opacity-50 text-[10px] uppercase">
                    <th>{i18n.t('Name')}</th>
                    <th class="hidden sm:table-cell">{i18n.t('Service')}/{i18n.t('Image')}</th>
                    <th class="text-center">{i18n.t('Ports')}</th>
                    <th class="text-center">{i18n.t('State')}</th>
                    <th class="text-right">{i18n.t('Actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {#each groupContainers as c (c.id)}
                    <tr class="hover" animate:flip={{ duration: 300 }}>
                      <td class="w-1/3">
                        <div class="font-bold truncate">{c.name}</div>
                        <div class="flex items-center gap-1 group">
                          <div class="text-[10px] opacity-50 font-mono">{c.id.slice(0, 12)}</div>
                          <button class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity" onclick={() => copyToClipboard(c.id)} title={i18n.t('Copy')}>📋</button>
                        </div>
                      </td>
                      <td class="hidden sm:table-cell opacity-70 italic text-xs truncate max-w-[150px]">{c.compose_service || c.image}</td>
                      <td class="text-center">
                        <span class="badge badge-xs {c.state === 'running' ? 'badge-success' : 'badge-ghost'}">{ c.ports.length ? c.ports.join(",") : i18n.t('NoPorts')}</span>
                      </td>
                      <td class="text-center">
                        <span class="badge badge-xs {c.state === 'running' ? 'badge-success' : 'badge-ghost'}">{c.state}</span>
                      </td>
                      <td class="text-right">
                        <div class="join">
                          {#if c.state !== 'running'}
                            <button class="btn btn-ghost btn-xs join-item" onclick={() => startContainer(c.id)}>▶️</button>
                          {:else}
                            <button class="btn btn-ghost btn-xs join-item" onclick={() => stopContainer(c.id)}>⏹️</button>
                          {/if}
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => openExec(c)} title={i18n.t('Exec')} disabled={c.state !== 'running'}>💻</button>
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => openFiles(c)} title={i18n.t('Files')} disabled={c.state !== 'running'}>📁</button>
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => openLogs(c)} title={i18n.t('Logs')}>📜</button>
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => openExport(c)} title={i18n.t('Export')}>📤</button>
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => inspectContainer(c)} title={i18n.t('Inspect')}>🔍</button>
                          <button class="btn btn-ghost btn-xs join-item text-error" onclick={() => { containerToRemove = c; showConfirmRemove = true; }} title={i18n.t('Remove')}>🗑️</button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      {/each}

      {#if groupedContainers.standalone.length > 0 || Object.keys(groupedContainers.groups).length === 0}
        <div class="bg-base-200 rounded-lg shadow min-h-[400px] flex flex-col h-[600px]">
          <table class="table table-zebra w-full table-fixed shrink-0">
            <thead>
              <tr>
                <th onclick={() => { sortCol = 'name'; sortDesc = !sortDesc; }} class="cursor-pointer w-1/3">{i18n.t('Name')} {sortCol === 'name' ? (sortDesc ? '▼' : '▲') : ''}</th>
                <th onclick={() => { sortCol = 'image'; sortDesc = !sortDesc; }} class="cursor-pointer hidden sm:table-cell w-1/4">{i18n.t('Image')} {sortCol === 'image' ? (sortDesc ? '▼' : '▲') : ''}</th>
                <th onclick={() => { sortCol = 'state'; sortDesc = !sortDesc; }} class="cursor-pointer w-1/6 text-center">{i18n.t('State')} {sortCol === 'state' ? (sortDesc ? '▼' : '▲') : ''}</th>
                <th class="w-1/4 text-right">{i18n.t('Actions')}</th>
              </tr>
            </thead>
          </table>

          <div class="flex-1 min-h-0">
            {#if groupedContainers.standalone.length === 0}
               <div class="text-center py-8 opacity-50">{i18n.t('NoContainersFound')}</div>
            {:else}
              <VirtualList items={groupedContainers.standalone} itemHeight={64} getKey={(c: DockerContainer) => c.id}>
                {#snippet children(c: DockerContainer)}
                  <div class="flex items-center px-4 h-full border-b border-base-content/5 hover:bg-base-300 transition-colors">
                    <div class="w-1/3 truncate">
                      <div class="font-bold truncate">{c.name}</div>
                      <div class="flex items-center gap-1 group">
                        <div class="text-xs opacity-50 font-mono hidden md:block">{c.id.slice(0, 12)}</div>
                        <button class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity hidden md:flex items-center justify-center" onclick={() => copyToClipboard(c.id)} title={i18n.t('Copy')}>📋</button>
                      </div>
                    </div>
                    <div class="hidden sm:block w-1/4 truncate opacity-70 italic text-xs">{c.image}</div>
                    <div class="w-1/6 text-center">
                      <span class="badge {c.state === 'running' ? 'badge-success' : 'badge-ghost'}">{c.state}</span>
                    </div>
                    <div class="w-1/4 text-right">
                      <div class="join">
                        {#if c.state !== 'running'}
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => startContainer(c.id)}>▶️</button>
                        {:else}
                          <button class="btn btn-ghost btn-xs join-item" onclick={() => stopContainer(c.id)}>⏹️</button>
                        {/if}
                        <button class="btn btn-ghost btn-xs join-item" onclick={() => openExec(c)} title={i18n.t('Exec')} disabled={c.state !== 'running'}>💻</button>
                        <button class="btn btn-ghost btn-xs join-item" onclick={() => openFiles(c)} title={i18n.t('Files')} disabled={c.state !== 'running'}>📁</button>
                        <button class="btn btn-ghost btn-xs join-item" onclick={() => openLogs(c)} title={i18n.t('Logs')}>📜</button>
                        <button class="btn btn-ghost btn-xs join-item" onclick={() => openExport(c)} title={i18n.t('Export')}>📤</button>
                        <button class="btn btn-ghost btn-xs join-item" onclick={() => inspectContainer(c)} title={i18n.t('Inspect')}>🔍</button>
                        <button class="btn btn-ghost btn-xs join-item text-error" onclick={() => { containerToRemove = c; showConfirmRemove = true; }} title={i18n.t('Remove')}>🗑️</button>
                      </div>
                    </div>
                  </div>
                {/snippet}
              </VirtualList>
            {/if}
          </div>
        </div>
      {/if}
    {/if}

    <div class="collapse bg-base-200 shadow-sm border border-base-300 shrink-0">
      <input type="checkbox" id="unused-images-collapse" onchange={(e) => e.currentTarget.checked && loadUnusedImages()} />
      <div class="collapse-title text-xl font-medium">{i18n.t('UnusedImages')}</div>
      <div class="collapse-content">
        <div class="overflow-x-auto">
          <table class="table table-sm">
            <thead>
              <tr>
                <th>{i18n.t('ImageID')}</th>
                <th>{i18n.t('Repository')}</th>
                <th>{i18n.t('Tag')}</th>
                <th>{i18n.t('Size')}</th>
              </tr>
            </thead>
            <tbody>
              {#each unusedImages as img (img.id)}
                <tr>
                  <td class="font-mono text-xs opacity-50">{img.id.slice(0, 12)}</td>
                  <td class="font-bold">{img.repository}</td>
                  <td><div class="badge badge-outline badge-xs">{img.tag}</div></td>
                  <td>{img.size}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</div>
