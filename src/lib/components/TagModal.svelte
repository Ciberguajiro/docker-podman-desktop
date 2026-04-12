<script lang="ts">
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { sanitize } from '$lib/utils';
  import type { CommandResult, DockerImage } from '$lib/types';

  let { show = $bindable(false), image, onComplete } = $props<{
    show: boolean;
    image: DockerImage | null;
    onComplete: () => void;
  }>();

  let repository = $state('');
  let tag = $state('latest');
  let isSubmitting = $state(false);

  $effect(() => {
    if (image) {
      repository = image.repository;
      tag = image.tag === '<none>' ? 'latest' : image.tag;
    }
  });

  async function handleSubmit() {
    if (!image || !repository) return;
    isSubmitting = true;
    const sanitizedRepo = sanitize(repository);
    const sanitizedTag = sanitize(tag);

    try {
      const res = await dockerStore.invoke<CommandResult>('docker_tag_image', {
        imageId: image.id,
        repository: sanitizedRepo,
        tag: sanitizedTag
      });
      if (res.success) {
        toastStore.success(`Image tagged as ${repository}:${tag}`);
        onComplete();
        show = false;
      } else {
        toastStore.error(`Error: ${res.error}`);
      }
    } catch (e) {
      toastStore.error(`Failed to tag image: ${e}`);
    } finally {
      isSubmitting = false;
    }
  }

  function close() {
    show = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show && image}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Tag Image</h3>
      <div class="py-4 space-y-4">
        <p class="text-sm opacity-70">Source: <span class="font-mono">{image.id.slice(0, 12)}</span></p>

        <div class="form-control w-full">
          <label class="label" for="tag-repo"><span class="label-text">Repository</span></label>
          <input id="tag-repo" type="text" placeholder="e.g. my-image" class="input input-bordered w-full" bind:value={repository} />
        </div>

        <div class="form-control w-full">
          <label class="label" for="tag-name"><span class="label-text">Tag</span></label>
          <input id="tag-name" type="text" placeholder="e.g. latest, v1.0" class="input input-bordered w-full" bind:value={tag} />
        </div>
      </div>
      <div class="modal-action">
        <button class="btn" onclick={close}>Cancel</button>
        <button class="btn btn-primary" onclick={handleSubmit} disabled={isSubmitting || !repository}>
          {#if isSubmitting}<span class="loading loading-spinner loading-xs"></span>{/if}
          Tag Image
        </button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
