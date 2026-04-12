<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import type { DockerVolume, CommandResult } from '$lib/types';

  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Card from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";

  import VolumeTable from './components/VolumeTable.svelte';
  import {
    RefreshCw,
    Plus,
    Trash2,
    Search,
    Database,
    HardDrive
  } from "lucide-svelte";
  import { cn } from '$lib/utils';

  let volumes = $state<DockerVolume[]>([]);
  let isLoading = $state(true);
  let searchQuery = $state('');
  let searchInput = $state('');
  let sortCol = $state('name');
  let sortDesc = $state(false);

  // Modals state
  let showCreateModal = $state(false);
  let newVolumeName = $state('');
  let showConfirmRemove = $state(false);
  let volumeToRemove = $state<DockerVolume | null>(null);
  let showConfirmPrune = $state(false);
  let showInspectModal = $state(false);
  let inspectData = $state('');

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  async function loadVolumes() {
    if (!dockerStore.selectedEngine) return;
    isLoading = true;
    try {
      volumes = await dockerService.getVolumes(dockerStore.selectedEngine);
    } catch (e) {
      console.error('Failed to load volumes', e);
      toastStore.error(`Failed to load volumes: ${e}`);
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
    let filtered = volumes.filter(v =>
      v.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      v.driver.toLowerCase().includes(searchQuery.toLowerCase())
    );

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'driver': res = a.driver.localeCompare(b.driver); break;
        case 'created': res = new Date(a.created).getTime() - new Date(b.created).getTime(); break;
        default: res = a.name.localeCompare(b.name);
      }
      return sortDesc ? -res : res;
    });
  });

  async function createVolume() {
    if (!newVolumeName || !dockerStore.selectedEngine) return;
    const res = await dockerService.createVolume(dockerStore.selectedEngine, newVolumeName);
    if (res.success) {
      toastStore.success(`Volume ${newVolumeName} created`);
      showCreateModal = false;
      newVolumeName = '';
      loadVolumes();
    } else {
      toastStore.error(`Error: ${res.error}`);
    }
  }

  async function removeVolume() {
    if (!volumeToRemove || !dockerStore.selectedEngine) return;
    const res = await dockerService.removeVolume(dockerStore.selectedEngine, volumeToRemove.name);
    if (res.success) toastStore.success(`Volume ${volumeToRemove.name} removed`);
    else toastStore.error(`Error: ${res.error}`);
    showConfirmRemove = false;
    loadVolumes();
  }

  async function pruneVolumes() {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.prune(dockerStore.selectedEngine, 'volumes');
    if (res.success) toastStore.success('Unused volumes pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    showConfirmPrune = false;
    loadVolumes();
  }

  async function inspectVolume(v: DockerVolume) {
    if (!dockerStore.selectedEngine) return;
    try {
      inspectData = await dockerService.inspect(dockerStore.selectedEngine, v.name);
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect volume: ${e}`);
    }
  }

  function toggleSort(col: string) {
    if (sortCol === col) sortDesc = !sortDesc;
    else {
      sortCol = col;
      sortDesc = false;
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
</script>

<div class="h-full flex flex-col bg-background">
  <!-- Header -->
  <header class="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-10">
    <div class="container flex flex-col md:flex-row md:items-center justify-between py-4 gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-primary/10 rounded-lg">
          <Database class="h-6 w-6 text-primary" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{i18n.t('Volumes')}</h1>
          <p class="text-xs text-muted-foreground">
            {volumes.length} {i18n.t('Volumes').toLowerCase()}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" onclick={loadVolumes} disabled={isLoading}>
          <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
          {i18n.t('Refresh')}
        </Button>
        <Button size="sm" onclick={() => showCreateModal = true}>
          <Plus class="h-4 w-4 mr-2" />
          {i18n.t('NewVolume')}
        </Button>
        <Button variant="destructive" size="sm" onclick={() => showConfirmPrune = true}>
          <Trash2 class="h-4 w-4 mr-2" />
          {i18n.t('Prune')}
        </Button>
      </div>
    </div>

    <!-- Filter -->
    <div class="container pb-4">
      <div class="relative max-w-md">
        <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          type="search"
          placeholder={i18n.t('Search') || 'Search volumes...'}
          class="pl-9"
          bind:value={searchInput}
        />
      </div>
    </div>
  </header>

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    <div class="container py-6">
      {#if isLoading && volumes.length === 0}
        <div class="grid gap-4">
          {#each Array(3) as i}
            <Card.Root class="h-24 animate-pulse bg-muted/20" />
          {/each}
        </div>
      {:else}
        <VolumeTable
          volumes={filteredVolumes}
          sortCol={sortCol}
          sortDesc={sortDesc}
          onSort={toggleSort}
          onCopy={copyToClipboard}
          onInspect={inspectVolume}
          onRemove={(v) => { volumeToRemove = v; showConfirmRemove = true; }}
        />

        {#if filteredVolumes.length === 0 && !isLoading}
          <div class="flex flex-col items-center justify-center py-20 text-center">
            <div class="bg-muted p-4 rounded-full mb-4">
              <HardDrive class="h-10 w-10 text-muted-foreground/50" />
            </div>
            <h3 class="text-lg font-medium">{i18n.t('NoVolumesFound') || 'No volumes found'}</h3>
            <p class="text-sm text-muted-foreground max-w-xs mt-1">
              Create a new volume to persist your data.
            </p>
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

<!-- Modals -->
{#if showCreateModal}
<ConfirmationModal
  title={i18n.t('NewVolume')}
  message=""
  onConfirm={createVolume}
  onCancel={() => showCreateModal = false}
>
  <div class="grid gap-2 py-4">
    <Label for="volume-name">{i18n.t('Name')}*</Label>
    <Input id="volume-name" bind:value={newVolumeName} placeholder="e.g. my-data-volume" />
  </div>
</ConfirmationModal>
{/if}

{#if showConfirmRemove && volumeToRemove}
<ConfirmationModal
  title={i18n.t('ConfirmRemove')}
  message={`${i18n.t('ConfirmRemoveMessage')} "${volumeToRemove.name}"?`}
  onConfirm={removeVolume}
  onCancel={() => showConfirmRemove = false}
  isDestructive={true}
  confirmText={i18n.t('Remove')}
/>
{/if}

{#if showConfirmPrune}
<ConfirmationModal
  title={i18n.t('ConfirmPrune')}
  message={i18n.t('ConfirmPruneMessage')}
  onConfirm={pruneVolumes}
  onCancel={() => showConfirmPrune = false}
  isDestructive={true}
  confirmText={i18n.t('Prune')}
/>
{/if}

{#if showInspectModal}
<InspectModal
  title="Inspect Volume"
  data={inspectData}
  onClose={() => showInspectModal = false}
/>
{/if}
