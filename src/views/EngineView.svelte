<script lang="ts">
  import type {
    AppConfig,
    AutomationStatus,
    CapabilitySummary,
    EngineProbe,
    TextModelProbe
  } from "../lib/types";

  export let config: AppConfig;
  export let capabilities: CapabilitySummary | null;
  export let engineProbe: EngineProbe | null;
  export let textModelProbe: TextModelProbe | null;
  export let automationStatus: AutomationStatus | null;
  export let busy: boolean;
  export let onSave: () => void;
  export let onProbe: () => void;
  export let onProbeTextModel: () => void;
  export let onStartAutomation: () => void;

  $: activeModel = config.engine.models.find(
    (model) => model.id === config.engine.active_model_id
  );
</script>

<section class="engine-layout">
  <div class="settings-panel">
    <div class="panel-intro">
      <p class="eyebrow">PRIMARY VIABILITY</p>
      <h2>Local inference engines</h2>
      <p>Image generation and creative planning are independent. Use stable-diffusion.cpp for pixels and a llama.cpp/OpenAI-compatible endpoint for copy, SVG direction, and prompts.</p>
    </div>

    <div class="settings-group first-group">
      <div class="section-heading">
        <div>
          <span>Image generation engine</span>
          <small>Normal images, backgrounds, web assets, product visuals, and LoRA workflows.</small>
        </div>
      </div>
      <div class="field-grid two">
        <label class="field">
          <span>Mode</span>
          <select bind:value={config.engine.mode}>
            <option value="mock">Mock viability engine</option>
            <option value="stable_diffusion_cpp">stable-diffusion.cpp</option>
          </select>
        </label>
        <label class="field">
          <span>Backend target</span>
          <select bind:value={config.engine.backend}>
            <option value="auto">Auto</option>
            <option value="vulkan">Vulkan / MoltenVK</option>
            <option value="metal">Metal</option>
            <option value="cpu">CPU fallback</option>
          </select>
        </label>
      </div>

      <label class="field">
        <span>sd-cli binary path</span>
        <input bind:value={config.engine.binary_path} placeholder="/Users/you/stable-diffusion.cpp/build/bin/sd-cli" />
      </label>

      {#if activeModel}
        <div class="nested-settings">
          <div class="section-heading">
            <div>
              <span>Primary image model profile</span>
              <small>Use either a complete model path or component paths for FLUX-style models.</small>
            </div>
          </div>
          <label class="field"><span>Profile label</span><input bind:value={activeModel.label} /></label>
          <label class="field"><span>Model family</span><input bind:value={activeModel.family} placeholder="flux, sdxl, sd3..." /></label>
          <label class="field"><span>Complete model path</span><input bind:value={activeModel.model_path} /></label>
          <label class="field"><span>Diffusion model path</span><input bind:value={activeModel.diffusion_model_path} /></label>
          <div class="field-grid two">
            <label class="field"><span>VAE path</span><input bind:value={activeModel.vae_path} /></label>
            <label class="field"><span>CLIP-L path</span><input bind:value={activeModel.clip_l_path} /></label>
          </div>
          <label class="field"><span>T5XXL / text encoder path</span><input bind:value={activeModel.t5xxl_path} /></label>
          <label class="field"><span>LoRA directory</span><input bind:value={activeModel.lora_directory} /></label>
          <label class="field">
            <span>Extra CLI arguments</span>
            <input
              value={activeModel.extra_args.join(" ")}
              onchange={(event) => {
                activeModel.extra_args = (event.currentTarget as HTMLInputElement).value
                  .split(" ")
                  .map((value) => value.trim())
                  .filter(Boolean);
              }}
              placeholder="--vae-tiling --clip-on-cpu"
            />
          </label>
        </div>
      {/if}
    </div>

    <div class="settings-group">
      <div class="section-heading">
        <div>
          <span>Local text + design model</span>
          <small>Works with llama-server and other OpenAI-compatible local endpoints.</small>
        </div>
      </div>
      <div class="field-grid two">
        <label class="field">
          <span>Provider</span>
          <select bind:value={config.text_model.mode}>
            <option value="mock">Mock creative planner</option>
            <option value="openai_compatible">OpenAI-compatible local server</option>
            <option value="disabled">Disabled</option>
          </select>
        </label>
        <label class="field">
          <span>Model name</span>
          <input bind:value={config.text_model.model} placeholder="local-model" />
        </label>
      </div>
      <label class="field">
        <span>Base URL</span>
        <input bind:value={config.text_model.base_url} placeholder="http://127.0.0.1:8080/v1" />
      </label>
      <label class="field">
        <span>API key</span>
        <input bind:value={config.text_model.api_key} placeholder="Optional for local servers" />
      </label>
      <div class="field-grid two">
        <label class="field">
          <span>Temperature</span>
          <input type="number" min="0" max="2" step="0.1" bind:value={config.text_model.temperature} />
        </label>
        <label class="field">
          <span>Max tokens</span>
          <input type="number" min="64" max="8192" step="64" bind:value={config.text_model.max_tokens} />
        </label>
      </div>
    </div>

    <div class="settings-group">
      <div class="section-heading">
        <div>
          <span>Automation API</span>
          <small>Authenticated and bound to 127.0.0.1 only.</small>
        </div>
      </div>
      <div class="field-grid two">
        <label class="field"><span>Port</span><input type="number" bind:value={config.automation.port} /></label>
        <label class="field"><span>Bearer token</span><input bind:value={config.automation.token} /></label>
      </div>
    </div>

    <div class="button-row wrap-buttons">
      <button class="primary-button" disabled={busy} onclick={onSave}>Save settings</button>
      <button class="secondary-button" disabled={busy} onclick={onProbe}>Probe image engine</button>
      <button class="secondary-button" disabled={busy} onclick={onProbeTextModel}>Probe text model</button>
      <button class="secondary-button" disabled={busy} onclick={onStartAutomation}>Start local API</button>
    </div>
  </div>

  <div class="diagnostics-panel">
    <div class="diagnostic-card">
      <span>Image generation</span>
      <strong>{capabilities?.image_generation ? "Ready" : "Not ready"}</strong>
    </div>
    <div class="diagnostic-card">
      <span>Local creative planning</span>
      <strong>{capabilities?.text_design_generation ? "Configured" : "Disabled"}</strong>
    </div>
    <div class="diagnostic-card">
      <span>LoRA schema</span>
      <strong>{capabilities?.loras ? "Ready" : "Not ready"}</strong>
    </div>
    <div class="diagnostic-card">
      <span>SVG composition</span>
      <strong>{capabilities?.svg_composition ? "Ready" : "Not ready"}</strong>
    </div>
    <div class="diagnostic-card muted-card">
      <span>Video inference</span>
      <strong>{capabilities?.video_generation ? "Ready" : "After P0 image tests"}</strong>
    </div>

    {#if engineProbe}
      <div class="terminal-card">
        <div><span class:terminal-ready={engineProbe.ready}></span>{engineProbe.summary}</div>
        <pre>{engineProbe.output}</pre>
      </div>
    {/if}

    {#if textModelProbe}
      <div class="terminal-card">
        <div><span class:terminal-ready={textModelProbe.ready}></span>{textModelProbe.summary}</div>
        <pre>{textModelProbe.output}</pre>
      </div>
    {/if}

    {#if automationStatus}
      <div class="automation-card">
        <span>Automation server</span>
        <strong>{automationStatus.address}</strong>
        <code>Authorization: Bearer {automationStatus.token}</code>
      </div>
    {/if}
  </div>
</section>
