<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import { sanitize, sanitizePorts } from '$lib/utils';
  import type { DockerImage, ImageHistoryEntry, CommandResult, DockerContainer } from '$lib/types';

  import ConfirmationModal from '$lib/components/ConfirmationModal.svelte';
  import InspectModal from '$lib/components/InspectModal.svelte';
  import PullModal from '$lib/components/PullModal.svelte';
  import TagModal from '$lib/components/TagModal.svelte';
  import BuildModal from '$lib/components/BuildModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Badge } from "$lib/components/ui/badge";
  import * as Card from "$lib/components/ui/card";
  import * as Table from "$lib/components/ui/table";
  import * as Select from "$lib/components/ui/select";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";

  import ImageTable from './components/ImageTable.svelte';
  import {
    RefreshCw,
    Download,
    Hammer,
    Upload,
    Trash2,
    Search,
    Image as ImageIcon,
    History as HistoryIcon,
    Play
  } from "lucide-svelte";

  let images = $state<DockerImage[]>([]);
  let isLoading = $state(true);
  let searchQuery = $state('');
  let searchInput = $state('');
  let sortCol = $state('repository');
  let sortDesc = $state(false);

  // Modals state
  let showPullModal = $state(false);
  let showBuildModal = $state(false);
  let showTagModal = $state(false);
  let selectedImage = $state<DockerImage | null>(null);
  let showConfirmRemove = $state(false);
  let imageToRemove = $state<DockerImage | null>(null);
  let showConfirmPrune = $state(false);
  let showInspectModal = $state(false);
  let inspectData = $state('');
  let showImportModal = $state(false);
  let importPath = $state('');
  let importRepo = $state('');
  let importTag = $state('');
  let showHistory = $state(false);
  let historyData = $state<ImageHistoryEntry[]>([]);
  let isHistoryLoading = $state(false);
  let showRunModal = $state(false);

  // Run container state
  let runImage = $state('');
  let runName = $state('');
  let runPorts = $state('');
  let runEnvs = $state('');
  let runRestartPolicy = $state('no');
  let runVolumes = $state('');

  $effect(() => {
    const timeout = setTimeout(() => {
      searchQuery = searchInput;
    }, 300);
    return () => clearTimeout(timeout);
  });

  async function loadImages() {
    if (!dockerStore.selectedEngine) return;
    isLoading = true;
    try {
      images = await dockerService.getImages(dockerStore.selectedEngine);
    } catch (e) {
      console.error('Failed to load images', e);
      toastStore.error(`Failed to load images: ${e}`);
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

  const filteredImages = $derived.by(() => {
    let filtered = images.filter(img =>
      img.repository.toLowerCase().includes(searchQuery.toLowerCase()) ||
      img.tag.toLowerCase().includes(searchQuery.toLowerCase()) ||
      img.id.toLowerCase().includes(searchQuery.toLowerCase())
    );

    return filtered.sort((a, b) => {
      let res = 0;
      switch (sortCol) {
        case 'repository': res = a.repository.localeCompare(b.repository); break;
        case 'tag': res = a.tag.localeCompare(b.tag); break;
        case 'id': res = a.id.localeCompare(b.id); break;
        case 'size': res = parseFloat(a.size) - parseFloat(b.size); break;
        case 'created': res = new Date(a.created).getTime() - new Date(b.created).getTime(); break;
        default: res = a.repository.localeCompare(b.repository);
      }
      return sortDesc ? -res : res;
    });
  });

  async function removeImage() {
    if (!imageToRemove || !dockerStore.selectedEngine) return;
    const res = await dockerService.removeImage(dockerStore.selectedEngine, imageToRemove.id);
    if (res.success) toastStore.success(`Image ${imageToRemove.repository} removed`);
    else toastStore.error(`Error: ${res.error}`);
    showConfirmRemove = false;
    loadImages();
  }

  async function pruneImages() {
    if (!dockerStore.selectedEngine) return;
    const res = await dockerService.prune(dockerStore.selectedEngine, 'images');
    if (res.success) toastStore.success('Unused images pruned');
    else toastStore.error(`Prune failed: ${res.error}`);
    showConfirmPrune = false;
    loadImages();
  }

  async function inspectImage(img: DockerImage) {
    if (!dockerStore.selectedEngine) return;
    try {
      inspectData = await dockerService.inspect(dockerStore.selectedEngine, img.id);
      showInspectModal = true;
    } catch (e) {
      toastStore.error(`Failed to inspect image: ${e}`);
    }
  }

  async function viewHistory(img: DockerImage) {
    if (!dockerStore.selectedEngine) return;
    selectedImage = img;
    showHistory = true;
    isHistoryLoading = true;
    try {
      historyData = await dockerService.getImageHistory(dockerStore.selectedEngine, img.id);
    } catch (e) {
      toastStore.error(`Failed to load history: ${e}`);
    } finally {
      isHistoryLoading = false;
    }
  }

  async function importImage() {
    if (!importPath || !importRepo || !dockerStore.selectedEngine) return;
    const res = await dockerService.importImage(dockerStore.selectedEngine, {
      path: importPath,
      repository: importRepo,
      tag: importTag || 'latest'
    });
    if (res.success) {
      toastStore.success(`Image ${importRepo} imported`);
      showImportModal = false;
      importPath = ''; importRepo = ''; importTag = '';
    } else {
      toastStore.error(`Import failed: ${res.error}`);
    }
    loadImages();
  }

  async function createContainer() {
    if (!runImage || !dockerStore.selectedEngine) return;
    const res = await dockerService.createContainer(dockerStore.selectedEngine, {
      image: sanitize(runImage),
      name: sanitize(runName),
      ports: sanitizePorts(runPorts),
      envs: sanitize(runEnvs),
      volumes: sanitize(runVolumes),
      restartPolicy: runRestartPolicy
    });
    if (res.success) {
      toastStore.success(`Container created from ${runImage}`);
      showRunModal = false;
      runImage = ''; runName = ''; runPorts = ''; runEnvs = ''; runVolumes = ''; runRestartPolicy = 'no';
    } else {
      toastStore.error(`Error: ${res.error}`);
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

  function openRun(img: DockerImage) {
    runImage = `${img.repository}:${img.tag}`;
    showRunModal = true;
  }

  function openTag(img: DockerImage) {
    selectedImage = img;
    showTagModal = true;
  }
</script>

<div class="h-full flex flex-col bg-background">
  <!-- Header -->
  <header class="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-10">
    <div class="container flex flex-col md:flex-row md:items-center justify-between py-4 gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-primary/10 rounded-lg">
          <ImageIcon class="h-6 w-6 text-primary" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{i18n.t('Images')}</h1>
          <p class="text-xs text-muted-foreground flex items-center gap-2">
            {images.length} {i18n.t('Images').toLowerCase()}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" onclick={loadImages} disabled={isLoading}>
          <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
          {i18n.t('Refresh')}
        </Button>
        <Button size="sm" variant="outline" onclick={() => showPullModal = true}>
          <Download class="h-4 w-4 mr-2" />
          {i18n.t('Pull')}
        </Button>
        <Button size="sm" variant="outline" onclick={() => showBuildModal = true}>
          <Hammer class="h-4 w-4 mr-2" />
          {i18n.t('Build')}
        </Button>
        <Button size="sm" variant="outline" onclick={() => showImportModal = true}>
          <Upload class="h-4 w-4 mr-2" />
          {i18n.t('Import')}
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
          placeholder={i18n.t('SearchImages')}
          class="pl-9"
          bind:value={searchInput}
        />
      </div>
    </div>
  </header>

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    <div class="container py-6">
      {#if isLoading && images.length === 0}
        <div class="grid gap-4">
          {#each Array(3) as _}
            <Card.Root class="h-24 animate-pulse bg-muted/20" />
          {/each}
        </div>
      {:else}
        <ImageTable
          images={filteredImages}
          sortCol={sortCol}
          sortDesc={sortDesc}
          onSort={toggleSort}
          onCopy={copyToClipboard}
          {onRun}
          {onTag}
          onInspect={inspectImage}
          onHistory={viewHistory}
          onRemove={(img) => { imageToRemove = img; showConfirmRemove = true; }}
        />

        {#if filteredImages.length === 0 && !isLoading}
          <div class="flex flex-col items-center justify-center py-20 text-center">
            <div class="bg-muted p-4 rounded-full mb-4">
              <ImageIcon class="h-10 w-10 text-muted-foreground/50" />
            </div>
            <h3 class="text-lg font-medium">{i18n.t('NoImagesFound')}</h3>
            <p class="text-sm text-muted-foreground max-w-xs mt-1">
              Pull or build an image to get started.
            </p>
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

<!-- Modals -->
{#if showPullModal}
<PullModal onClose={() => showPullModal = false} />
{/if}

{#if showBuildModal}
<BuildModal onClose={() => showBuildModal = false} />
{/if}

{#if showTagModal && selectedImage}
<TagModal
  imageId={selectedImage.id}
  repository={selectedImage.repository}
  tag={selectedImage.tag}
  onClose={() => showTagModal = false}
/>
{/if}

{#if showConfirmRemove && imageToRemove}
<ConfirmationModal
  title={i18n.t('Remove')}
  message={`${i18n.t('AreYouSurePrune')} ${imageToRemove.repository}:${imageToRemove.tag}?`}
  onConfirm={removeImage}
  onCancel={() => showConfirmRemove = false}
  isDestructive={true}
  confirmText={i18n.t('Remove')}
/>
{/if}

{#if showConfirmPrune}
<ConfirmationModal
  title={i18n.t('Prune')}
  message={i18n.t('AreYouSurePrune')}
  onConfirm={pruneImages}
  onCancel={() => showConfirmPrune = false}
  isDestructive={true}
  confirmText={i18n.t('Prune')}
/>
{/if}

{#if showInspectModal}
<InspectModal
  title="Inspect Image"
  data={inspectData}
  onClose={() => showInspectModal = false}
/>
{/if}

{#if showImportModal}
<ConfirmationModal
  title={i18n.t('Import')}
  message=""
  onConfirm={importImage}
  onCancel={() => showImportModal = false}
>
  <div class="grid gap-4 py-4">
    <div class="grid gap-2">
      <Label for="import-path">{i18n.t('Path')}*</Label>
      <div class="flex gap-2">
        <Input id="import-path" bind:value={importPath} placeholder="/path/to/image.tar" />
      </div>
    </div>
    <div class="grid gap-2">
      <Label for="import-repo">{i18n.t('Repository')}*</Label>
      <Input id="import-repo" bind:value={importRepo} placeholder="e.g. my-imported-image" />
    </div>
    <div class="grid gap-2">
      <Label for="import-tag">{i18n.t('Tag')}</Label>
      <Input id="import-tag" bind:value={importTag} placeholder="e.g. latest" />
    </div>
  </div>
</ConfirmationModal>
{/if}

{#if showRunModal}
<ConfirmationModal
  title={`${i18n.t('Run')} Image: ${runImage}`}
  message=""
  onConfirm={createContainer}
  onCancel={() => showRunModal = false}
  confirmText={i18n.t('Run')}
>
  <div class="grid gap-4 py-4">
    <div class="grid gap-2">
      <Label for="run-name">{i18n.t('ContainerName')}</Label>
      <Input id="run-name" bind:value={runName} placeholder="e.g. my-app" />
    </div>
    <div class="grid gap-2">
      <Label for="run-ports">{i18n.t('Ports')} (host:container)</Label>
      <Input id="run-ports" bind:value={runPorts} placeholder="e.g. 8080:80" />
    </div>
    <div class="grid gap-2">
      <Label for="run-envs">{i18n.t('EnvVars')}</Label>
      <Input id="run-envs" bind:value={runEnvs} placeholder="KEY=VAL, ..." />
    </div>
    <div class="grid gap-2">
      <Label for="run-restart">{i18n.t('RestartPolicy')}</Label>
      <Select.Root type="single" value={runRestartPolicy} onValueChange={v => runRestartPolicy = v || 'no'}>
        <Select.Trigger id="run-restart">
          {runRestartPolicy}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="no">no</Select.Item>
          <Select.Item value="always">always</Select.Item>
          <Select.Item value="on-failure">on-failure</Select.Item>
          <Select.Item value="unless-stopped">unless-stopped</Select.Item>
        </Select.Content>
      </Select.Root>
    </div>
    <div class="grid gap-2">
      <Label for="run-volumes">{i18n.t('Volumes')}</Label>
      <Input id="run-volumes" bind:value={runVolumes} placeholder="host:container, ..." />
    </div>
  </div>
</ConfirmationModal>
{/if}

{#if showHistory && selectedImage}
<ConfirmationModal
  title={`${i18n.t('History')}: ${selectedImage.repository}`}
  message=""
  onConfirm={() => showHistory = false}
  onCancel={() => showHistory = false}
  confirmText={i18n.t('Close')}
  showCancel={false}
>
  <div class="py-4">
    {#if isHistoryLoading}
      <div class="space-y-2">
        <div class="h-4 bg-muted animate-pulse rounded w-full"></div>
        <div class="h-4 bg-muted animate-pulse rounded w-3/4"></div>
      </div>
    {:else}
      <div class="rounded-md border overflow-hidden max-h-[400px] overflow-y-auto">
        <Table.Root>
          <Table.Header class="sticky top-0 bg-card z-10">
            <Table.Row>
              <Table.Head class="text-xs">{i18n.t('Created')}</Table.Head>
              <Table.Head class="text-xs">{i18n.t('CreatedBy')}</Table.Head>
              <Table.Head class="text-xs text-right">{i18n.t('Size')}</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each historyData as h (h.id)}
              <Table.Row>
                <Table.Cell class="text-[10px] whitespace-nowrap">{h.created}</Table.Cell>
                <Table.Cell class="max-w-[200px] truncate font-mono text-[10px]">{h.created_by}</Table.Cell>
                <Table.Cell class="text-right text-[10px]">{h.size}</Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      </div>
    {/if}
  </div>
</ConfirmationModal>
{/if}
