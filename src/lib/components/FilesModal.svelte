<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { ContainerFile, CommandResult } from '$lib/types';

  let { show = $bindable(false), containerId, containerName } = $props<{
    show: boolean;
    containerId: string;
    containerName: string;
  }>();

  let currentPath = $state('/');
  let files = $state<ContainerFile[]>([]);
  let isLoading = $state(false);
  let selectedFile = $state<ContainerFile | null>(null);
  let fileContent = $state('');
  let isEditing = $state(false);
  let isSaving = $state(false);

  async function loadFiles(path: string) {
    isLoading = true;
    try {
      files = await dockerStore.invoke<ContainerFile[]>('docker_container_ls', { containerId, path });
      currentPath = path;
      selectedFile = null;
      isEditing = false;
    } catch (e) {
      toastStore.error(`${i18n.t('Status')}: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  async function readFile(file: ContainerFile) {
    if (file.is_dir) {
      loadFiles(`${currentPath}${currentPath.endsWith('/') ? '' : '/'}${file.name}`);
      return;
    }

    isLoading = true;
    selectedFile = file;
    const fullPath = `${currentPath}${currentPath.endsWith('/') ? '' : '/'}${file.name}`;
    try {
      fileContent = await dockerStore.invoke<string>('docker_container_read_file', { containerId, path: fullPath });
      isEditing = true;
    } catch (e) {
      toastStore.error(`${i18n.t('Status')}: ${e}`);
      selectedFile = null;
    } finally {
      isLoading = false;
    }
  }

  async function saveFile() {
    if (!selectedFile) return;
    isSaving = true;
    const fullPath = `${currentPath}${currentPath.endsWith('/') ? '' : '/'}${selectedFile.name}`;
    try {
      const res = await dockerStore.invoke<CommandResult>('docker_container_write_file', {
        containerId,
        path: fullPath,
        content: fileContent
      });
      if (res.success) {
        toastStore.success(i18n.t('CopiedToClipboard') || 'File saved');
        isEditing = false;
        selectedFile = null;
      } else {
        toastStore.error(`Save failed: ${res.error}`);
      }
    } catch (e) {
      toastStore.error(`Error: ${e}`);
    } finally {
      isSaving = false;
    }
  }

  function goUp() {
    if (currentPath === '/') return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    loadFiles('/' + parts.join('/'));
  }

  $effect(() => {
    if (show && containerId) {
      loadFiles('/');
    }
  });

  function close() {
    show = false;
    files = [];
    currentPath = '/';
    selectedFile = null;
    isEditing = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal modal-open">
    <div class="modal-box w-11/12 max-w-5xl h-[80vh] flex flex-col">
      <div class="flex justify-between items-center mb-4 shrink-0">
        <h3 class="font-bold text-lg">{i18n.t('Files')} - {containerName}</h3>
        <div class="flex items-center gap-2">
          <div class="badge badge-outline font-mono text-xs">{currentPath}</div>
          <button class="btn btn-ghost btn-xs" onclick={() => loadFiles(currentPath)} disabled={isLoading}>
            {#if isLoading}
              <span class="loading loading-spinner loading-xs"></span>
            {/if}
            🔄
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-hidden flex gap-4 min-h-0">
        <!-- File List -->
        <div class="w-1/3 bg-base-300 rounded-lg flex flex-col">
          <div class="p-2 border-b border-base-content/10 flex justify-between items-center">
            <span class="text-xs font-bold uppercase opacity-50">{i18n.t('Explorer')}</span>
            <button class="btn btn-ghost btn-xs" onclick={goUp} disabled={currentPath === '/'}>⬆️</button>
          </div>
          <div class="flex-1 overflow-auto p-2 space-y-1">
            {#each files as file}
              <button
                class="w-full text-left px-2 py-1 rounded hover:bg-base-100 transition-colors flex items-center gap-2 text-sm {selectedFile?.name === file.name ? 'bg-primary text-primary-content' : ''}"
                onclick={() => readFile(file)}
              >
                <span>{file.is_dir ? '📁' : '📄'}</span>
                <span class="truncate">{file.name}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Editor / Preview -->
        <div class="flex-1 bg-base-200 rounded-lg flex flex-col overflow-hidden">
          {#if isEditing}
            <div class="p-2 border-b border-base-content/10 flex justify-between items-center shrink-0">
              <span class="text-xs font-mono truncate">{selectedFile?.name}</span>
              <div class="flex gap-2">
                <button class="btn btn-ghost btn-xs" onclick={() => { isEditing = false; selectedFile = null; }}>{i18n.t('Cancel')}</button>
                <button class="btn btn-primary btn-xs" onclick={saveFile} disabled={isSaving}>
                  {#if isSaving}<span class="loading loading-spinner loading-xs"></span>{/if}
                  💾 {i18n.t('Save')}
                </button>
              </div>
            </div>
            <textarea
              class="flex-1 p-4 font-mono text-xs bg-black text-green-400 focus:outline-none resize-none"
              bind:value={fileContent}
            ></textarea>
          {:else if isLoading && selectedFile}
             <div class="flex-1 flex items-center justify-center">
               <span class="loading loading-spinner loading-lg"></span>
             </div>
          {:else}
            <div class="flex-1 flex flex-col items-center justify-center opacity-30 text-center p-8">
              <div class="text-6xl mb-4">📂</div>
              <p>{i18n.t('SelectFileToView') || 'Select a file to view or edit its content'}</p>
            </div>
          {/if}
        </div>
      </div>

      <div class="modal-action shrink-0">
        <button class="btn" onclick={close}>{i18n.t('Close')}</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
