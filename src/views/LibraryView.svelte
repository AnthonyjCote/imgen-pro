<script lang="ts">
  import type { GeneratedAsset, GenerationJob } from "../lib/types";

  export let jobs: GenerationJob[];
  export let previews: Record<string, string>;
  export let onUseAsset: (asset: GeneratedAsset) => void;

  function useJobOutput(job: GenerationJob) {
    if (job.output) onUseAsset(job.output);
  }
</script>

<section class="library-section">
  <div class="section-title-row">
    <div>
      <p class="eyebrow">LOCAL FILE LIBRARY</p>
      <h2>Generated assets</h2>
    </div>
    <span>{jobs.filter((job) => job.output).length} saved</span>
  </div>

  {#if jobs.filter((job) => job.output).length === 0}
    <div class="empty-library">Generate an image to begin building the local asset library.</div>
  {:else}
    <div class="output-grid">
      {#each jobs.filter((job) => job.output) as job}
        <article class="output-card">
          <div class="asset-preview tall">
            {#if job.output && previews[job.output.path]}
              <img src={previews[job.output.path]} alt={job.request.prompt} />
            {/if}
          </div>
          <div class="output-meta">
            <strong>{job.request.width} × {job.request.height}</strong>
            <p>{job.request.prompt}</p>
            <code>{job.output?.path}</code>
            {#if job.output}
              <button class="text-button" onclick={() => useJobOutput(job)}>Open in composer →</button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
