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
    CreativePlan,
    DesignRequest,
    EngineProbe,
    GeneratedAsset,
    GenerationJob,
    GenerationRequest,
    ManagedImageServerStatus,
    TextModelProbe
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
  let imageServerStatus: ManagedImageServerStatus | null = null;
  let textModelProbe: TextModelProbe | null = null;
  let automationStatus: AutomationStatus | null = null;
  let renderedDesign: GeneratedAsset | null = null;
  let creativePlan: CreativePlan | null = null;
  let creativeBrief = "";
  let seedText = "";

  let generation: GenerationRequest = {
    prompt: "",
    negative_prompt: "blurry, distorted text, malformed product, low detail",
    model_id: "primary-model",
    width: 256,
    height: 256,
    steps: 1,
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
      await refreshImageServerStatus();
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

  async function generateCreativePlan() {
    if (!creativeBrief.trim()) {
      notice = "Add a creative brief before asking the local model to plan the design.";
      return;
    }

    busy = true;
    notice = "";
    try {
      creativePlan = await api.generateCreativePlan({
        brief: creativeBrief,
        template: design.template,
        width: design.width,
        height: design.height
      });
      design = {
        ...design,
        template: creativePlan.template,
        eyebrow: creativePlan.eyebrow,
        title: creativePlan.title,
        subtitle: creativePlan.subtitle,
        cta: creativePlan.cta
      };
      generation = {
        ...generation,
        prompt: creativePlan.image_prompt,
        negative_prompt: creativePlan.negative_prompt || generation.negative_prompt
      };
      notice = "Creative plan generated locally. The editable copy and image prompt are ready.";
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  function openPlanInImageGenerator() {
    view = "create";
    notice = "The local creative plan is loaded as a normal image-generation prompt.";
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
    design = { ...design, background_image_path: asset.path };
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

  async function runTextModelProbe() {
    busy = true;
    notice = "";
    try {
      textModelProbe = await api.probeTextModel();
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function refreshImageServerStatus() {
    try {
      imageServerStatus = await api.getImageServerStatus();
    } catch {
      imageServerStatus = null;
    }
  }

  async function startImageServer() {
    busy = true;
    notice = "";
    try {
      if (config) {
        config = await api.saveConfig(config);
        capabilities = await api.getCapabilities();
      }
      imageServerStatus = await api.startImageServer();
      notice = "Managed image server started.";
    } catch (error) {
      setError(error);
    } finally {
      busy = false;
    }
  }

  async function stopImageServer() {
    busy = true;
    notice = "";
    try {
      imageServerStatus = await api.stopImageServer();
      notice = "Managed image server stopped.";
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
    let unsubscribeJobs: (() => void) | undefined;
    let unsubscribeImageServer: (() => void) | undefined;

    void listen<string>("jobs://updated", () => {
      void refreshJobs();
    }).then((handler) => {
      unsubscribeJobs = handler;
    });

    void listen("image-server://updated", () => {
      void refreshImageServerStatus();
    }).then((handler) => {
      unsubscribeImageServer = handler;
    });

    return () => {
      unsubscribeJobs?.();
      unsubscribeImageServer?.();
    };
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
        {creativeBrief}
        {creativePlan}
        {renderedDesign}
        {previews}
        {busy}
        onCreativeBriefChange={(value) => (creativeBrief = value)}
        onGeneratePlan={() => void generateCreativePlan()}
        onOpenImageGenerator={openPlanInImageGenerator}
        onRender={() => void renderHybridDesign()}
      />
    {:else if view === "library"}
      <LibraryView {jobs} {previews} onUseAsset={useAssetInComposer} />
    {:else if config}
      <EngineView
        {config}
        {capabilities}
        {engineProbe}
        {imageServerStatus}
        {textModelProbe}
        {automationStatus}
        {busy}
        onSave={() => void saveEngineSettings()}
        onProbe={() => void runEngineProbe()}
        onProbeTextModel={() => void runTextModelProbe()}
        onStartImageServer={() => void startImageServer()}
        onStopImageServer={() => void stopImageServer()}
        onStartAutomation={() => void startAutomation()}
      />
    {/if}
  </main>
</div>
