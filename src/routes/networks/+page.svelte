<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '$lib/tauri';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import type { DockerNetwork, CommandResult } from '$lib/types';
  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';

  let networks = $state<DockerNetwork[]>([]);
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
  let newNetworkName = $state('');
  let newNetworkDriver = $state('bridge');

  let showConfirmRemove = $state(false);
  let networkToRemove = $state<DockerNetwork | null>(null);

  let showConfirmPrune = $state(false);

  let showInspectModal = $state(false);
  let inspectTitle = $state('');
  let inspectData = $state('');

  async function loadNetworks() {
    isLoading = true;
    try {
      networks = await dockerStore.invoke<DockerNetwork[]>('docker_networks');
      lastRefreshed = new Date();
    } catch (e) {
      console.error('Failed to load networks', e);
      toastStore.error(`${i18n.t('Networks')}: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadNetworks();
    }
  });

  onMount(() => {
    loadNetworks();
  });

  const filteredNetworks = $derived.by(() => {
    let filtered = networks.filter(n => !searchQuery || n.name.toLowerCase().includes(searchQuery.toLowerCase()));
    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'driver': res = a.driver.localeCompare(b.driver); break;
        case 'scope': res = a.scope.localeCompare(b.scope); break;
        case 'subnet': res = a.subnet.localeCompare(b.subnet); break;
        default: res = a.name.localeCompare(b.name);
      }
      return sortDesc ? -res : res;
    });
  });

  const paginatedNetworks = $derived.by(() => {
    const start = (currentPage - 1) * itemsPerPage;
    const end = start + itemsPerPage;
    return filteredNetworks.slice(start, end);
  });

  const totalPages = $derived(Math.ceil(filteredNetworks.length / itemsPerPage));

  async function createNetwork() {
    if (!newNetworkName) return;
    const res = await dockerStore.invoke<CommandResult>('docker_create_network', { name: newNetworkName, driver: newNetworkDriver });
    if (res.success) {
      toastStore.success(`${i18n.t('Networks')} ${newNetworkName} ${i18n.t('Created')}`);
      showCreateModal = false;
      newNetworkName = '';
    } else {
      toastStore.error(`${i18n.t('Status')}: ${res.error}`);
    }
    loadNetworks();
  }

  async function removeNetwork() {
    if (!networkToRemove) return;
    const res = await dockerStore.invoke<CommandResult>('docker_remove_network', { id: networkToRemove.id });
    if (res.success) toastStore.success(`${i18n.t('Networks')} ${networkToRemove.name} ${i18n.t('Remove')}`);
    else toastStore.error(`${i18n.t('Status')}: ${res.error}`);
    loadNetworks();
  }

  async function pruneNetworks() {
    const res = await dockerStore.invoke<CommandResult>('docker_prune', { type_: 'networks' });
    if (res.success) toastStore.success('Unused networks pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    loadNetworks();
  }

  async function inspectNetwork(n: DockerNetwork) {
    try {
      inspectTitle = `${i18n.t('InspectNetwork')}: ${n.name}`;
      inspectData = await dockerStore.invoke<string>('docker_inspect', { id: n.id });
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`${i18n.t('InspectNetwork')}: ${e}`);
    }
  }
</script>

<div class="p-6">
  <div class="flex justify-between items-center mb-6">
    <div class="flex items-center gap-4">
      <h1 class="text-3xl font-bold">{i18n.t('Networks')}</h1>
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
          <option value="scope">{i18n.t('Scope')}</option>
          <option value="subnet">{i18n.t('Subnet')}</option>
        </select>
        <button class="btn btn-sm join-item" onclick={() => sortDesc = !sortDesc}>
          {sortDesc ? '▼' : '▲'}
        </button>
      </div>
      <button class="btn btn-primary btn-sm" onclick={() => (showCreateModal = true)}>➕ {i18n.t('Create')}</button>
      <button class="btn btn-error btn-outline btn-sm" onclick={() => (showConfirmPrune = true)}>🧹 {i18n.t('Prune')}</button>
      <div class="flex flex-col items-end">
        <button class="btn btn-outline btn-sm" onclick={loadNetworks} disabled={isLoading}>
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
    title={i18n.t('RemoveNetwork')}
    message={networkToRemove ? i18n.t('RemoveNetworkMessage') : ''}
    onConfirm={removeNetwork}
  />

  <ConfirmationModal
    bind:show={showConfirmPrune}
    title={i18n.t('Prune')}
    message={i18n.t('AreYouSurePruneNetworks') || 'Are you sure you want to remove all unused networks?'}
    onConfirm={pruneNetworks}
  />

  <InspectModal
    bind:show={showInspectModal}
    title={inspectTitle}
    data={inspectData}
  />

  {#if showCreateModal}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">{i18n.t('CreateNetwork')}</h3>
        <div class="space-y-4 py-4">
          <input type="text" placeholder={i18n.t('NetworkName')} class="input input-bordered w-full" bind:value={newNetworkName} />
          <select class="select select-bordered w-full" bind:value={newNetworkDriver}>
            <option value="bridge">bridge</option>
            <option value="host">host</option>
            <option value="overlay">overlay</option>
            <option value="none">none</option>
          </select>
        </div>
        <div class="modal-action">
          <button class="btn" onclick={() => (showCreateModal = false)}>{i18n.t('Cancel')}</button>
          <button class="btn btn-primary" onclick={createNetwork}>{i18n.t('Create')}</button>
        </div>
      </div>
    </div>
  {/if}

  <div class="space-y-4">
    {#if isLoading && networks.length === 0}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each Array(6) as _}
          <div class="card bg-base-200 shadow-xl border border-base-300">
            <div class="card-body">
              <div class="skeleton h-6 w-3/4 mb-4"></div>
              <div class="space-y-2">
                <div class="skeleton h-3 w-1/2"></div>
                <div class="skeleton h-3 w-2/3"></div>
                <div class="skeleton h-3 w-3/4"></div>
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
      {#if filteredNetworks.length === 0}
        <div class="card bg-base-200 shadow-xl p-8 text-center opacity-50">
          {i18n.t('NoNetworksFound')}
        </div>
      {/if}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each paginatedNetworks as n (n.id)}
          <div class="card bg-base-200 shadow-xl border border-base-300">
            <div class="card-body">
              <h2 class="card-title text-accent truncate">{n.name}</h2>
              <div class="text-xs opacity-70">
                <p>{i18n.t('StorageDriver')}: {n.driver}</p>
                <p>{i18n.t('Scope')}: {n.scope}</p>
                <p>{i18n.t('Subnet')}: {n.subnet}</p>
              </div>
              <div class="card-actions justify-end mt-4">
                <button class="btn btn-ghost btn-sm" onclick={() => inspectNetwork(n)}>🔍 {i18n.t('Inspect')}</button>
                <button class="btn btn-error btn-outline btn-sm" onclick={() => { networkToRemove = n; showConfirmRemove = true; }}>🗑️</button>
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
