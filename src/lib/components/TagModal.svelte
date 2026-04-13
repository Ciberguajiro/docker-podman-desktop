<script lang="ts">
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { sanitize } from '$lib/utils';
  import type { CommandResult, DockerImage } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Tag, Loader2 } from "lucide-svelte";

  let { show = $bindable(true), image, onComplete } = $props<{
    show?: boolean;
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
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <Tag class="w-5 h-5 text-primary" />
        <Dialog.Title>Tag Image</Dialog.Title>
      </div>
      {#if image}
        <Dialog.Description>
          Create a new tag for image <code class="bg-muted px-1 rounded font-mono text-xs">{image.id.slice(0, 12)}</code>
        </Dialog.Description>
      {/if}
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="tag-repo">Repository</Label>
        <Input
          id="tag-repo"
          type="text"
          placeholder="e.g. my-image"
          bind:value={repository}
          disabled={isSubmitting}
        />
      </div>

      <div class="space-y-2">
        <Label for="tag-name">Tag</Label>
        <Input
          id="tag-name"
          type="text"
          placeholder="e.g. latest, v1.0"
          bind:value={tag}
          disabled={isSubmitting}
        />
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="ghost" onclick={close} disabled={isSubmitting}>
        Cancel
      </Button>
      <Button onclick={handleSubmit} disabled={isSubmitting || !repository}>
        {#if isSubmitting}
          <Loader2 class="w-4 h-4 mr-2 animate-spin" />
        {/if}
        Tag Image
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
