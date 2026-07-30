import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AutomationStatus,
  CapabilitySummary,
  CreativeBriefRequest,
  CreativePlan,
  DesignRequest,
  EngineProbe,
  GeneratedAsset,
  GenerationJob,
  GenerationRequest,
  ManagedImageServerStatus,
  TextModelProbe
} from "./types";

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<AppConfig>("save_config", { config }),
  getCapabilities: () => invoke<CapabilitySummary>("get_capabilities"),
  listJobs: () => invoke<GenerationJob[]>("list_jobs"),
  getJob: (id: string) => invoke<GenerationJob | null>("get_job", { id }),
  enqueueGeneration: (request: GenerationRequest) =>
    invoke<GenerationJob>("enqueue_generation", { request }),
  cancelJob: (id: string) => invoke<boolean>("cancel_job", { id }),
  probeEngine: () => invoke<EngineProbe>("probe_engine"),
  probeTextModel: () => invoke<TextModelProbe>("probe_text_model"),
  generateCreativePlan: (request: CreativeBriefRequest) =>
    invoke<CreativePlan>("generate_creative_plan", { request }),
  renderDesign: (request: DesignRequest) =>
    invoke<GeneratedAsset>("render_design", { request }),
  readAssetPreview: (path: string) => invoke<string>("read_asset_preview", { path }),
  startAutomationServer: () => invoke<AutomationStatus>("start_automation_server"),
  startImageServer: () => invoke<ManagedImageServerStatus>("start_image_server"),
  stopImageServer: () => invoke<ManagedImageServerStatus>("stop_image_server"),
  getImageServerStatus: () => invoke<ManagedImageServerStatus>("get_image_server_status")
};
