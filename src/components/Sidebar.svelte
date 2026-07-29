<script lang="ts">
  import type { AppConfig, CapabilitySummary, GenerationJob } from "../lib/types";

  export let view: "create" | "compose" | "library" | "engine";
  export let config: AppConfig | null;
  export let capabilities: CapabilitySummary | null;
  export let jobs: GenerationJob[];
  export let setView: (view: "create" | "compose" | "library" | "engine") => void;
</script>

<aside class="sidebar">
  <div class="brand-mark" aria-label="Imgen Pro">
    <span class="brand-symbol">IP</span>
    <div>
      <strong>Imgen Pro</strong>
      <small>Local creative engine</small>
    </div>
  </div>

  <nav aria-label="Primary navigation">
    <button class:active={view === "create"} onclick={() => setView("create")}>
      <span>01</span> Create
    </button>
    <button class:active={view === "compose"} onclick={() => setView("compose")}>
      <span>02</span> Compose
    </button>
    <button class:active={view === "library"} onclick={() => setView("library")}>
      <span>03</span> Library
    </button>
    <button class:active={view === "engine"} onclick={() => setView("engine")}>
      <span>04</span> Engine
    </button>
  </nav>

  <div class="sidebar-status">
    <div class="status-row">
      <span class="status-dot" class:ready={capabilities?.image_generation}></span>
      <div>
        <strong>{config?.engine.mode === "stable_diffusion_cpp" ? "Local engine" : "Mock engine"}</strong>
        <small>{config?.engine.backend ?? "Loading"} backend target</small>
      </div>
    </div>
    <div class="status-meta">
      <span>{jobs.filter((job) => job.status === "running").length} running</span>
      <span>{jobs.filter((job) => job.status === "queued").length} queued</span>
    </div>
  </div>
</aside>
