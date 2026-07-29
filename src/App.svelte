<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar from "./components/Sidebar.svelte";
  import Topbar from "./components/Topbar.svelte";
  import ComposeView from "./views/ComposeView.svelte";
  import CreateView from "./views/CreateView.svelte";
  import EngineView from "./views/EngineView.svelte";
  import LibraryView from "./views/LibraryView.svelte";
  import { api } from "./lib/api";
  import type {
    AppConfig,
    AutomationStatus,
    CapabilitySummary,
    DesignRequest,
    EngineProbe,
    GeneratedAsset,
    GenerationJob,
    GenerationRequest
  } from "./lib/types";

  type View = "create" | "compose" | "library" | "engine";

  let view: View = "create";
  let config: AppConfig | null = null;
  let capabilities: CapabilitySummary | null = null;
  let jobs: GenerationJob[] = [];
  let previews: Record<string, string> = {};
  let busy = false;
  let notice = "";
  let engineProbe: EngineProbe | null = null;
  let automationStatus: AutomationStatus | null = null;
  let renderedDesign: GeneratedAsset | null = null;
  let seedText = "";

  let generation: GenerationRequest = {
    prompt: "",
    negative_prompt: "blurry, distorted text, malformed product, low detail",
    model_id: "primary-model",
    width: 768,
    height: 768,
    steps: 4,
    seed: null,
    loras: []
  };

  let design: DesignRequest = {
    template: "feature-poster",
    eyebrow: "FEATURED CREATIVE",
    title: "Build a campaign, not just an image.",
    subtitle: "Combine local image generation with precise SVG typography and reusable marketing layouts.",
    cta: "Explore the launch",
    background_image_path: "",
    output_name: "imgen-pro-design",
    width: 1400,
    height: 900
  };

  async function initialize() {
    try {
      const [loadedConfig, loadedCapabilities, loadedJobs] = await Promise.all([
        api.getConfig(),
        api.getCapabilities(),
        api.listJobs()
      ]);
      config = loadedConfig;
      capabilities = loadedCapabilities;
      jobs = loadedJobs;
      generation.model_id = loadedConfig.engine.active_model_id;
      generation.steps =
        loadedConfig.engine.models.find((model) => model.id === loadedConfig.engine.active_model_id)
          ?.default_steps ?? generation.steps;
      await loadPreviews();
    } catch (error) {
      setError(error);
    }
  }

  async function refreshJobs() {
    jobs = await api.listJobs();
    await loadPreviews();
  }

  async function loadPreviews() {
    const paths = jobs
      .map((job) => job.output?.path)
      .filter((path): path is string => Boolean(path));

    if (renderedDesign?.path) paths.push(renderedDesign.path);

    for (const path of paths) {
      if (previews[path]) continue;
      try {
        previews[path] = await api.readAssetPreview(path);
        previews = { ...previews };
      } catch {
        // A generation event can arrive before the file is fully visible.
      }
    }
  }

  async function generateImage() {
    if (!generation.prompt.trim()) {
      notice = "Add a prompt before starting generation.";
      return;
    }

    busy = true;
    notice = "";
    try {
      generation.seed = seedText.trim() ? Number(seedText) : null;
      generation.model_id = config?.engine.active_model_id ?? generation.model_id;
      await api.enqueueGeneration(generation);
      await refreshJobs();
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function renderHybridDesign() {
    busy = true;
    notice = "";
    try {
      renderedDesign = await api.renderDesign(design);
      await loadPreviews();
      view = "compose";
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  function useAssetInComposer(asset: GeneratedAsset) {
    design.background_image_path = asset.path;
    view = "compose";
  }

  async function saveEngineSettings() {
    if (!config) return;
    busy = true;
    notice = "";
    try {
      config = await api.saveConfig(config);
      capabilities = await api.getCapabilities();
      generation.model_id = config.engine.active_model_id;
      notice = "Engine settings saved.";
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function runEngineProbe() {
    busy = true;
    notice = "";
    try {
      engineProbe = await api.probeEngine();
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function startAutomation() {
    busy = true;
    notice = "";
    try {
      automationStatus = await api.startAutomationServer();
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  function setError(error: unknown) {
    notice = error instanceof Error ? error.message : String(error);
  }

  onMount(() => {
    void initialize();
    let unsubscribe: (() => void) | undefined;

    void listen<string>("jobs://updated", () => {
      void refreshJobs();
    }).then((handler) => {
      unsubscribe = handler;
    });

    return () => unsubscribe?.();
  });
</script>

<svelte:head><title>Imgen Pro</title></svelte:head>

<div class="app-shell">
  <Sidebar {view} {config} {capabilities} {jobs} setView={(nextView) => (view = nextView)} />
  <main>
    <Topbar {view} refresh={() => void refreshJobs()} />
    {#if notice}<div class="notice">{notice}</div>{/if}

    {#if view === "create"}
      <CreateView
        {config}
        {generation}
        {jobs}
        {previews}
        {busy}
        {seedText}
        onSeedChange={(value) => (seedText = value)}
        onGenerate={() => void generateImage()}
        onUseAsset={useAssetInComposer}
      />
    {:else if view === "compose"}
      <ComposeView
        {design}
        {renderedDesign}
        {previews}
        {busy}
        onRender={() => void renderHybridDesign()}
      />
    {:else if view === "library"}
      <LibraryView {jobs} {previews} onUseAsset={useAssetInComposer} />
    {:else if config}
      <EngineView
        {config}
        {capabilities}
        {engineProbe}
        {automationStatus}
        {busy}
        onSave={() => void saveEngineSettings()}
        onProbe={() => void runEngineProbe()}
        onStartAutomation={() => void startAutomation()}
      />
    {/if}
  </main>
</div>
