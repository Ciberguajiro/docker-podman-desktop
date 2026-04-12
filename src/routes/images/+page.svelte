<script lang="ts">
  import { onMount } from 'svelte';
  import { flip } from 'svelte/animate';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import { sanitize, sanitizePorts, parseSize } from '$lib/utils';
  import type { DockerImage, CommandResult, ImageHistoryEntry } from '$lib/types';
  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import PullModal from '$lib/components/PullModal.svelte';
  import TagModal from '$lib/components/TagModal.svelte';
  import BuildModal from '$lib/components/BuildModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';
  import VirtualList from '$lib/components/VirtualList.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  let images = $state<DockerImage[]>([]);
  let isLoading = $state(true);
  let lastRefreshed = $state<Date | null>(null);
  let searchQuery = $state('');
  let searchInput = $state('');
  let danglingOnly = $state(false);

  let sortCol = $state('repository');
  let sortDesc = $state(false);

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  let showHistory = $state(false);
  let selectedImage = $state<DockerImage | null>(null);
  let historyData = $state<ImageHistoryEntry[]>([]);
  let isHistoryLoading = $state(false);

  let showConfirmRemove = $state(false);
  let imageToRemove = $state<DockerImage | null>(null);

  let showConfirmPrune = $state(false);

  let showImportModal = $state(false);
  let importPath = $state('');
  let importRepo = $state('');
  let importTag = $state('');

  let showPullModal = $state(false);
  let showBuildModal = $state(false);
  let showTagModal = $state(false);
  let imageToTag = $state<DockerImage | null>(null);

  let showInspectModal = $state(false);
  let inspectTitle = $state('');
  let inspectData = $state('');

  let showRunModal = $state(false);
  let runImage = $state('');
  let runName = $state('');
  let runPorts = $state('');
  let runEnvs = $state('');
  let runVolumes = $state('');
  let runRestartPolicy = $state('no');

  async function loadImages() {
    isLoading = true;
    try {
      images = await dockerStore.invoke<DockerImage[]>('docker_images');
      lastRefreshed = new Date();
    } catch (e) {
      console.error('Failed to load images', e);
      toastStore.error('Failed to load images');
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadImages();
    }
  });

  onMount(() => {
    loadImages();
  });

  const allImageNames = $derived(images.map(img => img.tag === '<none>' ? img.id : `${img.repository}:${img.tag}`));

  const filteredImages = $derived.by(() => {
    let filtered = images.filter(img => {
      const matchesQuery = !searchQuery ||
        img.repository.toLowerCase().includes(searchQuery.toLowerCase()) ||
        img.tag.toLowerCase().includes(searchQuery.toLowerCase()) ||
        img.id.toLowerCase().includes(searchQuery.toLowerCase());
      const isDangling = img.tag === '<none>';
      return matchesQuery && (!danglingOnly || isDangling);
    });

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'repository': res = a.repository.localeCompare(b.repository); break;
        case 'tag': res = a.tag.localeCompare(b.tag); break;
        case 'id': res = a.id.localeCompare(b.id); break;
        case 'size': res = parseSize(a.size) - parseSize(b.size); break;
        case 'created': res = a.created.localeCompare(b.created); break;
        default: res = a.repository.localeCompare(b.repository);
      }
      return sortDesc ? -res : res;
    });
  });

  $effect(() => {
    // Reset when filters change
    searchQuery;
    danglingOnly;
  });

  async function viewHistory(img: DockerImage) {
    selectedImage = img;
    showHistory = true;
    isHistoryLoading = true;
    try {
      historyData = await dockerStore.invoke<ImageHistoryEntry[]>('docker_image_history', { imageId: img.id });
    } catch (e) {
      console.error('Failed to load history', e);
      toastStore.error('Failed to load history');
    } finally {
      isHistoryLoading = false;
    }
  }

  async function removeImage() {
    if (!imageToRemove) return;
    const res = await dockerStore.invoke<CommandResult>('docker_remove_image', { imageId: imageToRemove.id, force: false });
    if (res.success) toastStore.success(`Image ${imageToRemove.repository} removed`);
    else toastStore.error(`Error: ${res.error}`);
    loadImages();
  }

  async function pruneImages() {
    const res = await dockerStore.invoke<CommandResult>('docker_prune', { type_: 'images' });
    if (res.success) toastStore.success('Unused images pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    loadImages();
  }

  async function inspectImage(img: DockerImage) {
    try {
      inspectTitle = `Inspect Image: ${img.repository}:${img.tag}`;
      inspectData = await dockerStore.invoke<string>('docker_inspect', { id: img.id });
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect image: ${e}`);
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

  function openRun(img: DockerImage) {
    runImage = img.tag === '<none>' ? img.id : `${img.repository}:${img.tag}`;
    runName = '';
    runPorts = '';
    runEnvs = '';
    runVolumes = '';
    showRunModal = true;
  }

  async function selectImportFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Docker Image', extensions: ['tar'] }]
    });
    if (selected && !Array.isArray(selected)) {
      importPath = selected;
    }
  }

  async function importImage() {
    if (!importPath || !importRepo) return;
    const res = await dockerStore.invoke<CommandResult>('docker_import_image', {
      path: importPath,
      repository: sanitize(importRepo),
      tag: sanitize(importTag)
    });
    if (res.success) {
      toastStore.success(`Image imported from ${importPath}`);
      showImportModal = false;
      importPath = ''; importRepo = ''; importTag = '';
      loadImages();
    } else {
      toastStore.error(`Import failed: ${res.error}`);
    }
  }

  async function createContainer() {
    if (!runImage) return;
    const sanitizedImage = sanitize(runImage);
    const sanitizedName = sanitize(runName);
    const sanitizedPorts = sanitizePorts(runPorts);
    const sanitizedEnvs = sanitize(runEnvs);
    const sanitizedVolumes = sanitize(runVolumes);

    const res = await dockerStore.invoke<CommandResult>('docker_create_container', {
      image: sanitizedImage,
      name: sanitizedName,
      ports: sanitizedPorts,
      envs: sanitizedEnvs,
      volumes: sanitizedVolumes,
      restartPolicy: runRestartPolicy
    });
    if (res.success) {
      toastStore.success(`Container created from ${runImage}`);
      showRunModal = false;
      runRestartPolicy = 'no';
    } else {
      toastStore.error(`Error: ${res.error}`);
    }
  }

  function openTag(img: DockerImage) {
    imageToTag = img;
    showTagModal = true;
  }

  function setSort(col: string) {
    if (sortCol === col) sortDesc = !sortDesc;
    else {
      sortCol = col;
      sortDesc = false;
    }
  }
</script>

<div class="p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-6 shrink-0">
    <div class="flex items-center gap-4">
      <h1 class="text-3xl font-bold">{i18n.t('Images')}</h1>
      <div class="relative">
        <input
          type="text"
          placeholder={i18n.t('Search')}
          class="input input-bordered input-sm w-64"
          bind:value={searchInput}
        />
        <span class="absolute right-2 top-1.5 opacity-30">🔍</span>
      </div>
      <div class="form-control">
        <label class="label cursor-pointer gap-2">
          <span class="label-text">{i18n.t('Dangling')}</span>
          <input type="checkbox" class="checkbox checkbox-primary checkbox-sm" bind:checked={danglingOnly} />
        </label>
      </div>
    </div>
    <div class="flex items-center gap-4">
      <button class="btn btn-primary btn-sm" onclick={() => (showImportModal = true)}>📥 {i18n.t('Import')}</button>
      <button class="btn btn-primary btn-sm btn-outline" onclick={() => (showPullModal = true)}>☁️ {i18n.t('Pull')}</button>
      <button class="btn btn-primary btn-sm btn-outline" onclick={() => (showBuildModal = true)}>🛠️ {i18n.t('Build')}</button>
      <button class="btn btn-error btn-outline btn-sm" onclick={() => (showConfirmPrune = true)}>🧹 {i18n.t('Prune')}</button>
      <div class="flex flex-col items-end">
        <button class="btn btn-outline btn-sm" onclick={loadImages} disabled={isLoading}>
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

  <PullModal
    bind:show={showPullModal}
    onComplete={loadImages}
  />

  <BuildModal
    bind:show={showBuildModal}
    onComplete={loadImages}
  />

  <TagModal
    bind:show={showTagModal}
    image={imageToTag}
    onComplete={loadImages}
  />

  <InspectModal
    bind:show={showInspectModal}
    title={inspectTitle}
    data={inspectData}
  />

  {#if showImportModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('Import')} Image</h3>
        <div class="space-y-4 py-4">
          <div class="form-control w-full">
            <label class="label" for="import-path"><span class="label-text">{i18n.t('InputPath')} (.tar) *</span></label>
            <div class="flex gap-2">
              <input id="import-path" type="text" placeholder="e.g. /path/to/image.tar" class="input input-bordered w-full" bind:value={importPath} readonly />
              <button class="btn btn-square btn-outline" onclick={selectImportFile}>📂</button>
            </div>
          </div>
          <div class="form-control w-full">
            <label class="label" for="import-repo"><span class="label-text">{i18n.t('Repository')} *</span></label>
            <input id="import-repo" type="text" placeholder="e.g. my-imported-image" class="input input-bordered w-full" bind:value={importRepo} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="import-tag"><span class="label-text">{i18n.t('Tag')}</span></label>
            <input id="import-tag" type="text" placeholder="e.g. latest" class="input input-bordered w-full" bind:value={importTag} />
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showImportModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={importImage} disabled={!importPath || !importRepo}>{i18n.t('Import')}</button>
        </div>
      </div>
      <div class="modal-backdrop" role="presentation" onclick={() => (showImportModal = false)}></div>
    </div>
  {/if}

  <ConfirmationModal
    bind:show={showConfirmRemove}
    title={i18n.t('Remove')}
    message={imageToRemove ? `${i18n.t('AreYouSurePrune')} ${imageToRemove.repository}:${imageToRemove.tag}?` : ''}
    onConfirm={removeImage}
  />

  <ConfirmationModal
    bind:show={showConfirmPrune}
    title={i18n.t('Prune')}
    message={i18n.t('AreYouSurePrune')}
    onConfirm={pruneImages}
  />

  {#if showRunModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('Run')} Image: {runImage}</h3>
        <div class="space-y-4 py-4">
          <div class="form-control w-full">
            <label class="label" for="run-image"><span class="label-text">{i18n.t('ImageName')} *</span></label>
            <input id="run-image" type="text" class="input input-bordered w-full" bind:value={runImage} list="run-image-suggestions" />
            <datalist id="run-image-suggestions">
              {#each allImageNames as name}
                <option value={name}></option>
              {/each}
            </datalist>
          </div>
          <div class="form-control w-full">
            <label class="label" for="run-name"><span class="label-text">{i18n.t('ContainerName')}</span></label>
            <input id="run-name" type="text" placeholder="e.g. my-app" class="input input-bordered w-full" bind:value={runName} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="run-ports"><span class="label-text">{i18n.t('Ports')} (host:container)</span></label>
            <input id="run-ports" type="text" placeholder="e.g. 8080:80" class="input input-bordered w-full" bind:value={runPorts} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="run-envs"><span class="label-text">{i18n.t('EnvVars')} (KEY=VAL, ...)</span></label>
            <input id="run-envs" type="text" placeholder="e.g. NODE_ENV=production,PORT=3000" class="input input-bordered w-full" bind:value={runEnvs} />
          </div>
          <div class="form-control w-full">
            <label class="label" for="run-restart-policy"><span class="label-text">{i18n.t('RestartPolicy')}</span></label>
            <select id="run-restart-policy" class="select select-bordered w-full" bind:value={runRestartPolicy}>
              <option value="no">no</option>
              <option value="always">always</option>
              <option value="on-failure">on-failure</option>
              <option value="unless-stopped">unless-stopped</option>
            </select>
          </div>
          <div class="form-control w-full">
            <label class="label" for="run-volumes"><span class="label-text">{i18n.t('VolumesMapping')} (host:container, ...)</span></label>
            <input id="run-volumes" type="text" placeholder="e.g. /host/path:/container/path, myvol:/data" class="input input-bordered w-full" bind:value={runVolumes} />
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showRunModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={createContainer}>{i18n.t('Run')}</button>
        </div>
      </div>
      <div class="modal-backdrop" role="presentation" onclick={() => (showRunModal = false)}></div>
    </div>
  {/if}

  {#if showHistory}
    <div class="modal modal-open">
      <div class="modal-box w-11/12 max-w-5xl">
        <h3 class="font-bold text-lg">{i18n.t('History')}: {selectedImage?.repository}</h3>
        <div class="py-4">
          {#if isHistoryLoading}
            <div class="skeleton h-24 w-full"></div>
          {:else}
            <div class="overflow-x-auto">
              <table class="table table-xs">
                <thead>
                  <tr>
                    <th>{i18n.t('Created')}</th>
                    <th>{i18n.t('CreatedBy')}</th>
                    <th>{i18n.t('Size')}</th>
                  </tr>
                </thead>
                <tbody>
                  {#each historyData as h (h.id)}
                    <tr>
                      <td>{h.created}</td>
                      <td class="max-w-md truncate font-mono">{h.created_by}</td>
                      <td>{h.size}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showHistory = false)}>{i18n.t('Close')}</button>
        </div>
      </div>
      <div class="modal-backdrop" role="presentation" onclick={() => (showHistory = false)}></div>
    </div>
  {/if}

  <div class="space-y-4 flex-1 flex flex-col min-h-0">
    {#if isLoading && images.length === 0}
      <div class="card bg-base-200 shadow-xl p-4 space-y-4">
        <div class="skeleton h-8 w-full"></div>
        <div class="skeleton h-12 w-full"></div>
      </div>
    {:else if images.length === 0}  
    <div class="card bg-base-200 shadow-xl p-4 space-y-4 shrink-0">
        <p>{i18n.t('NoImagesFound')}</p>
      </div>
    {:else}
      <div class="bg-base-200 rounded-lg shadow flex flex-col flex-1 min-h-0">
        <table class="table table-zebra w-full table-fixed shrink-0">
          <thead>
            <tr>
              <th onclick={() => setSort('repository')} class="cursor-pointer w-1/4">{i18n.t('Repository')} {sortCol === 'repository' ? (sortDesc ? '▼' : '▲') : ''}</th>
              <th onclick={() => setSort('tag')} class="cursor-pointer hidden md:table-cell w-1/6">{i18n.t('Tag')} {sortCol === 'tag' ? (sortDesc ? '▼' : '▲') : ''}</th>
              <th onclick={() => setSort('id')} class="cursor-pointer hidden lg:table-cell w-1/6">{i18n.t('ImageID')} {sortCol === 'id' ? (sortDesc ? '▼' : '▲') : ''}</th>
              <th onclick={() => setSort('size')} class="cursor-pointer w-1/12">{i18n.t('Size')} {sortCol === 'size' ? (sortDesc ? '▼' : '▲') : ''}</th>
              <th onclick={() => setSort('created')} class="cursor-pointer hidden sm:table-cell w-1/6">{i18n.t('Created')} {sortCol === 'created' ? (sortDesc ? '▼' : '▲') : ''}</th>
              <th class="w-1/6 text-right">{i18n.t('Actions')}</th>
            </tr>
          </thead>
        </table>

        <div class="flex-1 min-h-0">
          {#if filteredImages.length === 0}
            <div class="text-center py-8 opacity-50">{i18n.t('NoImagesFound')}</div>
          {:else}
            <VirtualList items={filteredImages} itemHeight={64} getKey={(img: DockerImage) => img.id}>
              {#snippet children(img: DockerImage)}
                <div class="flex items-center px-4 h-full border-b border-base-content/5 hover:bg-base-300 transition-colors">
                  <div class="w-1/4 truncate font-bold text-primary">{img.repository}</div>
                  <div class="hidden md:block w-1/6">
                    <span class="badge badge-outline">{img.tag}</span>
                  </div>
                  <div class="hidden lg:block w-1/6 group">
                    <div class="flex items-center gap-1">
                      <div class="font-mono text-xs opacity-50">{img.id.slice(0, 12)}</div>
                      <button class="btn btn-ghost btn-xs p-0 min-h-0 h-4 w-4 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center" onclick={() => copyToClipboard(img.id)} title={i18n.t('Copy')}>📋</button>
                    </div>
                  </div>
                  <div class="w-1/12">{img.size}</div>
                  <div class="hidden sm:block w-1/6 text-xs opacity-70">{img.created}</div>
                  <div class="w-1/6 text-right">
                    <div class="join">
                      <button class="btn btn-ghost btn-xs join-item" onclick={() => openRun(img)} title={i18n.t('Run')}>▶️</button>
                      <button class="btn btn-ghost btn-xs join-item" onclick={() => openTag(img)} title={i18n.t('Tag')}>🏷️</button>
                      <button class="btn btn-ghost btn-xs join-item" onclick={() => inspectImage(img)} title={i18n.t('Inspect')}>🔍</button>
                      <button class="btn btn-ghost btn-xs join-item" onclick={() => viewHistory(img)} title={i18n.t('History')}>📜</button>
                      <button class="btn btn-ghost btn-xs join-item text-error" onclick={() => { imageToRemove = img; showConfirmRemove = true; }} title={i18n.t('Remove')}>🗑️</button>
                    </div>
                  </div>
                </div>
              {/snippet}
            </VirtualList>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
