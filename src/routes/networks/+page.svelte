<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import type { DockerNetwork, CommandResult } from '$lib/types';

  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Card from "$lib/components/ui/card";
  import * as Select from "$lib/components/ui/select";
  import { Label } from "$lib/components/ui/label";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import Container from "$lib/components/ui/Container.svelte";

  import NetworkTable from './components/NetworkTable.svelte';
  import {
    RefreshCw,
    Plus,
    Trash2,
    Search,
    Network as NetworkIcon,
    Globe
  } from "lucide-svelte";
  import { cn } from '$lib/utils';

  let networks = $state<DockerNetwork[]>([]);
  let isLoading = $state(true);
  let searchQuery = $state('');
  let searchInput = $state('');
  let sortCol = $state('name');
  let sortDesc = $state(false);

  // Modals state
  let showCreateModal = $state(false);
  let newNetworkName = $state('');
  let newNetworkDriver = $state('bridge');
  let showConfirmRemove = $state(false);
  let networkToRemove = $state<DockerNetwork | null>(null);
  let showConfirmPrune = $state(false);
  let showInspectModal = $state(false);
  let inspectData = $state('');

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  async function loadNetworks() {
    if (!dockerStore.selectedEngine) return;
    isLoading = true;
    try {
      networks = await dockerService.getNetworks(dockerStore.selectedEngine);
    } catch (e) {
      console.error('Failed to load networks', e);
      toastStore.error(`Failed to load networks: ${e}`);
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
    let filtered = networks.filter(n =>
      n.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      n.driver.toLowerCase().includes(searchQuery.toLowerCase())
    );

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'name': res = a.name.localeCompare(b.name); break;
        case 'driver': res = a.driver.localeCompare(b.driver); break;
        case 'scope': res = a.scope.localeCompare(b.scope); break;
        default: res = a.name.localeCompare(b.name);
      }
      return sortDesc ? -res : res;
    });
  });

  async function createNetwork() {
    if (!newNetworkName || !dockerStore.selectedEngine) return;
    const res = await dockerService.createNetwork(dockerStore.selectedEngine, newNetworkName, newNetworkDriver);
    if (res.success) {
      toastStore.success(`Network ${newNetworkName} created`);
      showCreateModal = false;
      newNetworkName = '';
      loadNetworks();
    } else {
      toastStore.error(`Error: ${res.error}`);
    }
  }

  async function removeNetwork() {
    if (!networkToRemove || !dockerStore.selectedEngine) return;
    const res = await dockerService.removeNetwork(dockerStore.selectedEngine, networkToRemove.id);
    if (res.success) toastStore.success(`Network ${networkToRemove.name} removed`);
    else toastStore.error(`Error: ${res.error}`);
    showConfirmRemove = false;
    loadNetworks();
  }

  async function pruneNetworks() {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.prune(dockerStore.selectedEngine, 'networks');
    if (res.success) toastStore.success('Unused networks pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    showConfirmPrune = false;
    loadNetworks();
  }

  async function inspectNetwork(n: DockerNetwork) {
    if (!dockerStore.selectedEngine) return;
    try {
      inspectData = await dockerService.inspect(dockerStore.selectedEngine, n.id);
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect network: ${e}`);
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

<Container>
  <PageHeader
    title={i18n.t('Networks')}
    description="Manage and configure virtual networks for container communication."
    icon={Globe}
  >
    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" onclick={loadNetworks} disabled={isLoading}>
        <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
        {i18n.t('Refresh')}
      </Button>
      <Button size="sm" onclick={() => showCreateModal = true}>
        <Plus class="h-4 w-4 mr-2" />
        {i18n.t('NewNetwork')}
      </Button>
      <Button variant="destructive" size="sm" onclick={() => showConfirmPrune = true}>
        <Trash2 class="h-4 w-4 mr-2" />
        {i18n.t('Prune')}
      </Button>
    </div>
  </PageHeader>

  <div class="relative max-w-md w-full">
    <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
    <Input
      type="search"
      placeholder={i18n.t('SearchNetworks')}
      class="pl-9"
      bind:value={searchInput}
    />
  </div>

  <div class="space-y-6">
    {#if isLoading && networks.length === 0}
      <div class="grid gap-4">
        {#each Array(3) as _}
          <Card.Root class="h-24 animate-pulse bg-muted/20" />
        {/each}
      </div>
    {:else}
      <NetworkTable
        networks={filteredNetworks}
        sortCol={sortCol}
        sortDesc={sortDesc}
        onSort={toggleSort}
        onCopy={copyToClipboard}
        onInspect={inspectNetwork}
        onRemove={(n) => { networkToRemove = n; showConfirmRemove = true; }}
      />

      {#if filteredNetworks.length === 0 && !isLoading}
        <div class="flex flex-col items-center justify-center py-20 text-center">
          <div class="bg-muted p-4 rounded-full mb-4">
            <NetworkIcon class="h-10 w-10 text-muted-foreground/50" />
          </div>
          <h3 class="text-lg font-medium">{i18n.t('NoNetworksFound')}</h3>
          <p class="text-sm text-muted-foreground max-w-xs mt-1">
            Create a new network to connect your containers.
          </p>
        </div>
      {/if}
    {/if}
  </div>
</Container>

<!-- Modals -->
{#if showCreateModal}
<ConfirmationModal
  title={i18n.t('NewNetwork')}
  message=""
  onConfirm={createNetwork}
  onCancel={() => showCreateModal = false}
>
  <div class="grid gap-4 py-4">
    <div class="grid gap-2">
      <Label for="network-name">{i18n.t('Name')}*</Label>
      <Input id="network-name" bind:value={newNetworkName} placeholder="e.g. my-app-network" />
    </div>
    <div class="grid gap-2">
      <Label for="network-driver">{i18n.t('Driver')}</Label>
      <Select.Root type="single" value={newNetworkDriver} onValueChange={v => newNetworkDriver = v || 'bridge'}>
        <Select.Trigger id="network-driver">
          {newNetworkDriver}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="bridge">bridge</Select.Item>
          <Select.Item value="host">host</Select.Item>
          <Select.Item value="overlay">overlay</Select.Item>
          <Select.Item value="macvlan">macvlan</Select.Item>
          <Select.Item value="none">none</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>
  </div>
</ConfirmationModal>
{/if}

{#if showConfirmRemove && networkToRemove}
<ConfirmationModal
  title={i18n.t('ConfirmRemove')}
  message={`${i18n.t('ConfirmRemoveMessage')} "${networkToRemove.name}"?`}
  onConfirm={removeNetwork}
  onCancel={() => showConfirmRemove = false}
  isDestructive={true}
  confirmText={i18n.t('Remove')}
/>
{/if}

{#if showConfirmPrune}
<ConfirmationModal
  title={i18n.t('ConfirmPrune')}
  message={i18n.t('ConfirmPruneMessage')}
  onConfirm={pruneNetworks}
  onCancel={() => showConfirmPrune = false}
  isDestructive={true}
  confirmText={i18n.t('Prune')}
/>
{/if}

{#if showInspectModal}
<InspectModal
  title="Inspect Network"
  data={inspectData}
  bind:show={showInspectModal}
/>
{/if}
