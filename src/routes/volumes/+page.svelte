<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '$lib/tauri';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { DockerVolume, CommandResult } from '$lib/types';
  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';

  let volumes = $state<DockerVolume[]>([]);
  let isLoading = $state(true);
  let lastRefreshed = $state<Date | null>(null);
  let searchQuery = $state('');
  let searchInput = $state('');
  let sortCol = $state('name');
  let sortDesc = $state(false);

  let currentPage = $state(1);
  const itemsPerPage = 12;

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  $effect(() => {
    // Reset to page 1 when search change
    searchQuery;
    currentPage = 1;
  });

  let showCreateModal = $state(false);
  let newVolumeName = $state('');

  let showConfirmRemove = $state(false);
  let volumeToRemove = $state<DockerVolume | null>(null);

  let showConfirmPrune = $state(false);

  let showInspectModal = $state(false);
  let inspectTitle = $state('');
  let inspectData = $state('');

  async function loadVolumes() {
    isLoading = true;
    try {
      volumes = await dockerStore.invoke<DockerVolume[]>('docker_volumes');
      lastRefreshed = new Date();
    } catch (e) {
      console.error('Failed to load volumes', e);
      toastStore.error(`${i18n.t('Volumes')}: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadVolumes();
    }
  });

  onMount(() => {
    loadVolumes();
  });

  const filteredVolumes = $derived.by(() => {
    let filtered = volumes.filter(v => !searchQuery || v.name.toLowerCase().includes(searchQuery.toLowerCase()));
    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'driver': res = a.driver.localeCompare(b.driver); break;
        case 'created': res = a.created.localeCompare(b.created); break;
        default: res = a.name.localeCompare(b.name);
      }
      return sortDesc ? -res : res;
    });
  });

  const paginatedVolumes = $derived.by(() => {
    const start = (currentPage - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return filteredVolumes.slice(start, end);
  });

  const totalPages = $derived(Math.ceil(filteredVolumes.length / itemsPerPage));

  async function createVolume() {
    if (!newVolumeName) return;
    const res = await dockerStore.invoke<CommandResult>('docker_create_volume', { name: newVolumeName });
    if (res.success) {
      toastStore.success(`${i18n.t('Volumes')} ${newVolumeName} ${i18n.t('Created')}`);
      showCreateModal = false;
      newVolumeName = '';
    } else {
      toastStore.error(`${i18n.t('Status')}: ${res.error}`);
    }
    loadVolumes();
  }

  async function removeVolume() {
    if (!volumeToRemove) return;
    const res = await dockerStore.invoke<CommandResult>('docker_remove_volume', { name: volumeToRemove.name });
    if (res.success) toastStore.success(`${i18n.t('Volumes')} ${volumeToRemove.name} ${i18n.t('Remove')}`);
    else toastStore.error(`${i18n.t('Status')}: ${res.error}`);
    loadVolumes();
  }

  async function pruneVolumes() {
    const res = await dockerStore.invoke<CommandResult>('docker_prune', { type_: 'volumes' });
    if (res.success) toastStore.success('Unused volumes pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    loadVolumes();
  }

  async function inspectVolume(v: DockerVolume) {
    try {
      inspectTitle = `${i18n.t('InspectVolume')}: ${v.name}`;
      inspectData = await dockerStore.invoke<string>('docker_inspect', { id: v.name });
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`${i18n.t('InspectVolume')}: ${e}`);
    }
  }
</script>

<div class="p-6">
  <div class="flex justify-between items-center mb-6">
    <div class="flex items-center gap-4">
      <h1 class="text-3xl font-bold">{i18n.t('Volumes')}</h1>
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
      <div class="join">
        <select class="select select-bordered select-sm join-item" bind:value={sortCol}>
          <option value="name">{i18n.t('Name')}</option>
          <option value="driver">{i18n.t('StorageDriver')}</option>
          <option value="created">{i18n.t('Created')}</option>
        </select>
        <button class="btn btn-sm join-item" onclick={() => sortDesc = !sortDesc}>
          {sortDesc ? '▼' : '▲'}
        </button>
      </div>
      <button class="btn btn-primary btn-sm" onclick={() => (showCreateModal = true)}>➕ {i18n.t('Create')}</button>
      <button class="btn btn-error btn-outline btn-sm" onclick={() => (showConfirmPrune = true)}>🧹 {i18n.t('Prune')}</button>
      <div class="flex flex-col items-end">
        <button class="btn btn-outline btn-sm" onclick={loadVolumes} disabled={isLoading}>
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
    title={i18n.t('RemoveVolume')}
    message={volumeToRemove ? i18n.t('RemoveVolumeMessage') : ''}
    onConfirm={removeVolume}
  />

  <ConfirmationModal
    bind:show={showConfirmPrune}
    title={i18n.t('Prune')}
    message={i18n.t('AreYouSurePruneVolumes') || 'Are you sure you want to remove all unused volumes?'}
    onConfirm={pruneVolumes}
  />

  <InspectModal
    bind:show={showInspectModal}
    title={inspectTitle}
    data={inspectData}
  />

  {#if showCreateModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('CreateVolume')}</h3>
        <div class="py-4">
          <input type="text" placeholder={i18n.t('VolumeName')} class="input input-bordered w-full" bind:value={newVolumeName} />
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showCreateModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={createVolume}>{i18n.t('Create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="space-y-4">
    {#if isLoading && volumes.length === 0}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each Array(6) as _}
          <div class="card bg-base-200 shadow-xl border border-base-300">
            <div class="card-body">
              <div class="skeleton h-6 w-3/4 mb-4"></div>
              <div class="space-y-2">
                <div class="skeleton h-3 w-1/2"></div>
                <div class="skeleton h-3 w-2/3"></div>
              </div>
              <div class="card-actions justify-end mt-4">
                <div class="skeleton h-8 w-20"></div>
                <div class="skeleton h-8 w-8"></div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      {#if filteredVolumes.length === 0}
        <div class="card bg-base-200 shadow-xl p-8 text-center opacity-50">
          {i18n.t('NoVolumesFound')}
        </div>
      {/if}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each paginatedVolumes as v (v.name)}
          <div class="card bg-base-200 shadow-xl border border-base-300">
            <div class="card-body">
              <h2 class="card-title text-secondary truncate">{v.name}</h2>
              <div class="text-xs opacity-70">
                <p>{i18n.t('StorageDriver')}: {v.driver}</p>
                <p>{i18n.t('Created')}: {v.created}</p>
              </div>
              <div class="card-actions justify-end mt-4">
                <button class="btn btn-ghost btn-sm" onclick={() => inspectVolume(v)}>🔍 {i18n.t('Inspect')}</button>
                <button class="btn btn-error btn-outline btn-sm" onclick={() => { volumeToRemove = v; showConfirmRemove = true; }}>🗑️</button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    {#if totalPages > 1}
      <div class="flex justify-center mt-4">
        <div class="join">
          <button
            class="join-item btn btn-sm"
            disabled={currentPage === 1}
            onclick={() => currentPage--}
          >«</button>
          <button class="join-item btn btn-sm">Page {currentPage} of {totalPages}</button>
          <button
            class="join-item btn btn-sm"
            disabled={currentPage === totalPages}
            onclick={() => currentPage++}
          >»</button>
        </div>
      </div>
    {/if}
  </div>
</div>
