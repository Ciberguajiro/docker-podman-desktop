<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { ContainerFile, CommandResult } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Folder,
    FileText,
    ArrowUp,
    RefreshCw,
    Save,
    X,
    FolderTree,
    Loader2
  } from "lucide-svelte";
  import { cn } from "$lib/utils";

  let { show = $bindable(true), containerId, containerName } = $props<{
    show?: boolean;
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
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[1000px] h-[85vh] flex flex-col">
    <Dialog.Header>
      <div class="flex items-center justify-between mr-8">
        <div class="flex items-center gap-2">
          <FolderTree class="w-5 h-5 text-primary" />
          <Dialog.Title>{i18n.t('Files')} - {containerName}</Dialog.Title>
        </div>
        <div class="flex items-center gap-2">
          <Badge variant="outline" class="font-mono text-[10px] py-0 px-2 h-6">{currentPath}</Badge>
          <Button variant="ghost" size="icon" class="h-8 w-8" onclick={() => loadFiles(currentPath)} disabled={isLoading}>
            {#if isLoading}
              <Loader2 class="w-3.5 h-3.5 animate-spin" />
            {:else}
              <RefreshCw class="w-3.5 h-3.5" />
            {/if}
          </Button>
        </div>
      </div>
    </Dialog.Header>

    <div class="flex-1 overflow-hidden flex gap-4 min-h-0 py-4">
      <!-- File List -->
      <div class="w-1/3 bg-muted/50 rounded-lg border flex flex-col overflow-hidden">
        <div class="p-2 bg-muted border-b flex justify-between items-center shrink-0">
          <span class="text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-1">{i18n.t('Explorer')}</span>
          <Button variant="ghost" size="icon" class="h-6 w-6" onclick={goUp} disabled={currentPath === '/'}>
            <ArrowUp class="w-3 h-3" />
          </Button>
        </div>
        <div class="flex-1 overflow-auto p-1 space-y-0.5">
          {#each files as file}
            <button
              class={cn(
                "w-full text-left px-2 py-1.5 rounded-md transition-colors flex items-center gap-2 text-xs",
                selectedFile?.name === file.name
                  ? "bg-primary text-primary-foreground shadow-sm font-medium"
                  : "hover:bg-muted"
              )}
              onclick={() => readFile(file)}
            >
              {#if file.is_dir}
                <Folder class={cn("w-3.5 h-3.5", selectedFile?.name === file.name ? "text-primary-foreground" : "text-blue-500")} />
              {:else}
                <FileText class={cn("w-3.5 h-3.5", selectedFile?.name === file.name ? "text-primary-foreground" : "text-muted-foreground")} />
              {/if}
              <span class="truncate">{file.name}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Editor / Preview -->
      <div class="flex-1 bg-muted/30 rounded-lg border flex flex-col overflow-hidden shadow-inner">
        {#if isEditing}
          <div class="p-2 bg-muted/50 border-b flex justify-between items-center shrink-0">
            <div class="flex items-center gap-2 px-1">
              <FileText class="w-3 h-3 text-primary" />
              <span class="text-[10px] font-mono truncate max-w-[200px]">{selectedFile?.name}</span>
            </div>
            <div class="flex gap-1">
              <Button variant="ghost" size="sm" class="h-7 text-[10px] px-2" onclick={() => { isEditing = false; selectedFile = null; }}>
                <X class="w-3 h-3 mr-1" />
                {i18n.t('Cancel')}
              </Button>
              <Button size="sm" class="h-7 text-[10px] px-2" onclick={saveFile} disabled={isSaving}>
                {#if isSaving}
                  <Loader2 class="w-3 h-3 mr-1 animate-spin" />
                {:else}
                  <Save class="w-3 h-3 mr-1" />
                {/if}
                {i18n.t('Save')}
              </Button>
            </div>
          </div>
          <textarea
            class="flex-1 p-4 font-mono text-[11px] leading-relaxed bg-transparent focus:outline-none resize-none"
            bind:value={fileContent}
          ></textarea>
        {:else if isLoading && selectedFile}
             <div class="flex-1 flex flex-col items-center justify-center opacity-40">
               <Loader2 class="w-10 h-10 animate-spin mb-4" />
               <p class="text-sm">Reading file content...</p>
             </div>
        {:else}
          <div class="flex-1 flex flex-col items-center justify-center opacity-30 text-center p-8">
            <div class="p-4 bg-muted rounded-full mb-4">
              <Folder class="w-12 h-12 text-muted-foreground" />
            </div>
            <h4 class="text-sm font-semibold mb-1">Preview</h4>
            <p class="text-xs max-w-[200px]">{i18n.t('SelectFileToView') || 'Select a file to view or edit its content'}</p>
          </div>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="outline" onclick={close}>{i18n.t('Close')}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
