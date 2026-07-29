<script lang="ts">
  import type {
    AppConfig,
    GeneratedAsset,
    GenerationJob,
    GenerationRequest
  } from "../lib/types";

  export let config: AppConfig | null;
  export let generation: GenerationRequest;
  export let jobs: GenerationJob[];
  export let previews: Record<string, string>;
  export let busy: boolean;
  export let seedText: string;
  export let onSeedChange: (value: string) => void;
  export let onGenerate: () => void;
  export let onUseAsset: (asset: GeneratedAsset) => void;

  function useJobOutput(job: GenerationJob) {
    if (job.output) onUseAsset(job.output);
  }
</script>

<section class="generator-layout">
  <div class="control-panel">
    <div class="mode-tabs" aria-label="Generation mode">
      <button class="selected">Image</button>
      <button disabled title="Video adapter follows image viability testing">Video <span>Soon</span></button>
    </div>

    <label class="field prompt-field">
      <span>Describe your image</span>
      <textarea
        bind:value={generation.prompt}
        rows="8"
        placeholder="A cinematic product poster for a premium oatmeal stout, offset can, cold condensation, dramatic bowling lane lighting..."
      ></textarea>
      <small>Use the mock engine to validate the workflow before loading a multi-gigabyte model.</small>
    </label>

    <label class="field">
      <span>Negative prompt</span>
      <textarea bind:value={generation.negative_prompt} rows="3"></textarea>
    </label>

    <div class="field-grid two">
      <label class="field">
        <span>Model</span>
        <select bind:value={generation.model_id} disabled={!config}>
          {#each config?.engine.models ?? [] as model}
            <option value={model.id}>{model.label}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span>Seed</span>
        <input
          value={seedText}
          oninput={(event) => onSeedChange((event.currentTarget as HTMLInputElement).value)}
          inputmode="numeric"
          placeholder="Random"
        />
      </label>
    </div>

    <div class="field-grid three">
      <label class="field">
        <span>Width</span>
        <input type="number" min="256" max="2048" step="64" bind:value={generation.width} />
      </label>
      <label class="field">
        <span>Height</span>
        <input type="number" min="256" max="2048" step="64" bind:value={generation.height} />
      </label>
      <label class="field">
        <span>Steps</span>
        <input type="number" min="1" max="100" bind:value={generation.steps} />
      </label>
    </div>

    <div class="lora-section">
      <div class="section-heading">
        <div>
          <span>LoRA stack</span>
          <small>Profiles become selectable after paths are added in Engine.</small>
        </div>
        <span class="count-pill">{config?.engine.loras.length ?? 0}</span>
      </div>
      {#if (config?.engine.loras.length ?? 0) === 0}
        <div class="empty-inline">No LoRA profiles configured yet.</div>
      {:else}
        {#each config?.engine.loras ?? [] as lora}
          <label class="lora-row">
            <input
              type="checkbox"
              checked={generation.loras.some((selection) => selection.id === lora.id)}
              onchange={(event) => {
                const checked = (event.currentTarget as HTMLInputElement).checked;
                generation.loras = checked
                  ? [...generation.loras, { id: lora.id, strength: 0.8 }]
                  : generation.loras.filter((selection) => selection.id !== lora.id);
              }}
            />
            <span>{lora.label}</span>
          </label>
        {/each}
      {/if}
    </div>

    <button class="primary-button" disabled={busy} onclick={onGenerate}>
      {busy ? "Working…" : "Generate locally"}
    </button>
  </div>

  <div class="result-panel">
    <div class="result-heading">
      <div>
        <p class="eyebrow">LATEST OUTPUTS</p>
        <h2>Creative queue</h2>
      </div>
      <span>{jobs.length} jobs</span>
    </div>

    {#if jobs.length === 0}
      <div class="hero-empty">
        <div class="empty-orb"></div>
        <h3>Your first output appears here.</h3>
        <p>Start in mock mode to verify job orchestration, file output, previews, and automation before installing a model.</p>
      </div>
    {:else}
      <div class="output-grid compact">
        {#each jobs.slice(0, 6) as job}
          <article class="output-card">
            <div class="asset-preview">
              {#if job.output && previews[job.output.path]}
                <img src={previews[job.output.path]} alt={job.request.prompt} />
              {:else}
                <div class="generation-state">
                  <span class:spinning={job.status === "running"}></span>
                  <strong>{job.status}</strong>
                  <small>{job.progress}%</small>
                </div>
              {/if}
            </div>
            <div class="output-meta">
              <div>
                <span class="status-chip status-{job.status}">{job.status}</span>
                <strong>{job.request.model_id}</strong>
              </div>
              <p>{job.request.prompt}</p>
              {#if job.output}
                <button class="text-button" onclick={() => useJobOutput(job)}>Use in design →</button>
              {/if}
              {#if job.error_message}<small class="error-text">{job.error_message}</small>{/if}
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </div>
</section>
