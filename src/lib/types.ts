export type EngineMode = "mock" | "stable_diffusion_cpp" | "stable_diffusion_cpp_server";
export type BackendPreference = "auto" | "vulkan" | "metal" | "cpu";
export type TextProviderMode = "mock" | "disabled" | "openai_compatible";
export type JobStatus = "queued" | "running" | "completed" | "failed" | "cancelled";
export type AssetKind = "image" | "video" | "design";

export interface LoraProfile {
  id: string;
  label: string;
  path: string;
  trigger: string;
  compatible_families: string[];
}

export interface ModelProfile {
  id: string;
  label: string;
  family: string;
  enabled: boolean;
  model_path: string;
  diffusion_model_path: string;
  vae_path: string;
  clip_l_path: string;
  t5xxl_path: string;
  llm_path: string;
  lora_directory: string;
  extra_args: string[];
  default_steps: number;
}

export interface EngineConfig {
  mode: EngineMode;
  binary_path: string;
  server_binary_path: string;
  server_url: string;
  backend: BackendPreference;
  active_model_id: string;
  models: ModelProfile[];
  loras: LoraProfile[];
}

export interface TextModelConfig {
  mode: TextProviderMode;
  base_url: string;
  model: string;
  api_key: string;
  temperature: number;
  max_tokens: number;
}

export interface AutomationConfig {
  enabled: boolean;
  port: number;
  token: string;
}

export interface AppConfig {
  engine: EngineConfig;
  text_model: TextModelConfig;
  automation: AutomationConfig;
}

export interface LoraSelection {
  id: string;
  strength: number;
}

export interface GenerationRequest {
  prompt: string;
  negative_prompt: string;
  model_id: string;
  width: number;
  height: number;
  steps: number;
  seed: number | null;
  loras: LoraSelection[];
}

export interface GeneratedAsset {
  id: string;
  path: string;
  mime_type: string;
  width: number;
  height: number;
  kind: AssetKind;
}

export interface GenerationJob {
  id: string;
  request: GenerationRequest;
  status: JobStatus;
  progress: number;
  phase: string;
  elapsed_seconds: number | null;
  eta_seconds: number | null;
  created_at: string;
  updated_at: string;
  output: GeneratedAsset | null;
  error_message: string | null;
  logs: string[];
}

export interface EngineProbe {
  ready: boolean;
  mode: EngineMode;
  binary_path: string;
  summary: string;
  output: string;
}

export interface ManagedImageServerStatus {
  running: boolean;
  pid: number | null;
  address: string;
  phase: string;
  logs: string[];
}

export interface TextModelProbe {
  ready: boolean;
  base_url: string;
  summary: string;
  output: string;
}

export interface CreativeBriefRequest {
  brief: string;
  template: string;
  width: number;
  height: number;
}

export interface CreativePlan {
  template: string;
  eyebrow: string;
  title: string;
  subtitle: string;
  cta: string;
  image_prompt: string;
  negative_prompt: string;
}

export interface DesignRequest {
  template: string;
  title: string;
  subtitle: string;
  cta: string;
  eyebrow: string;
  background_image_path: string;
  output_name: string;
  width: number;
  height: number;
}

export interface AutomationStatus {
  running: boolean;
  address: string;
  token: string;
}

export interface CapabilitySummary {
  image_generation: boolean;
  video_generation: boolean;
  loras: boolean;
  svg_composition: boolean;
  text_design_generation: boolean;
  automation_api: boolean;
  engine_mode: EngineMode;
}
