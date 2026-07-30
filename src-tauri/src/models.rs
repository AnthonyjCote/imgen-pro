use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineMode {
    Mock,
    StableDiffusionCpp,
    StableDiffusionCppServer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendPreference {
    Auto,
    Vulkan,
    Metal,
    Cpu,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextProviderMode {
    Mock,
    Disabled,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraProfile {
    pub id: String,
    pub label: String,
    pub path: String,
    pub trigger: String,
    pub compatible_families: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub id: String,
    pub label: String,
    pub family: String,
    pub enabled: bool,
    pub model_path: String,
    pub diffusion_model_path: String,
    pub vae_path: String,
    pub clip_l_path: String,
    pub t5xxl_path: String,
    #[serde(default)]
    pub llm_path: String,
    pub lora_directory: String,
    pub extra_args: Vec<String>,
    pub default_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub mode: EngineMode,
    pub binary_path: String,
    #[serde(default)]
    pub server_binary_path: String,
    #[serde(default = "default_sd_server_url")]
    pub server_url: String,
    pub backend: BackendPreference,
    pub active_model_id: String,
    pub models: Vec<ModelProfile>,
    pub loras: Vec<LoraProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextModelConfig {
    pub mode: TextProviderMode,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for TextModelConfig {
    fn default() -> Self {
        Self {
            mode: TextProviderMode::Mock,
            base_url: "http://127.0.0.1:8080/v1".to_string(),
            model: "local-model".to_string(),
            api_key: String::new(),
            temperature: 0.7,
            max_tokens: 700,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub enabled: bool,
    pub port: u16,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub engine: EngineConfig,
    #[serde(default)]
    pub text_model: TextModelConfig,
    pub automation: AutomationConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            engine: EngineConfig {
                mode: EngineMode::Mock,
                binary_path: String::new(),
                server_binary_path: String::new(),
                server_url: default_sd_server_url(),
                backend: BackendPreference::Vulkan,
                active_model_id: "primary-model".to_string(),
                models: vec![ModelProfile {
                    id: "primary-model".to_string(),
                    label: "Primary local model".to_string(),
                    family: "flux".to_string(),
                    enabled: true,
                    model_path: String::new(),
                    diffusion_model_path: String::new(),
                    vae_path: String::new(),
                    clip_l_path: String::new(),
                    t5xxl_path: String::new(),
                    llm_path: String::new(),
                    lora_directory: String::new(),
                    extra_args: Vec::new(),
                    default_steps: 4,
                }],
                loras: Vec::new(),
            },
            text_model: TextModelConfig::default(),
            automation: AutomationConfig {
                enabled: false,
                port: 4762,
                token: Uuid::new_v4().to_string(),
            },
        }
    }
}

fn default_sd_server_url() -> String {
    "http://127.0.0.1:1234".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraSelection {
    pub id: String,
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: String,
    pub model_id: String,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub seed: Option<u64>,
    #[serde(default)]
    pub loras: Vec<LoraSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Image,
    Video,
    Design,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAsset {
    pub id: String,
    pub path: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub kind: AssetKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    pub id: String,
    pub request: GenerationRequest,
    pub status: JobStatus,
    pub progress: u8,
    #[serde(default = "default_job_phase")]
    pub phase: String,
    #[serde(default)]
    pub elapsed_seconds: Option<u64>,
    #[serde(default)]
    pub eta_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub output: Option<GeneratedAsset>,
    pub error_message: Option<String>,
    pub logs: Vec<String>,
}

fn default_job_phase() -> String {
    "Queued".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineProbe {
    pub ready: bool,
    pub mode: EngineMode,
    pub binary_path: String,
    pub summary: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedImageServerStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub address: String,
    pub phase: String,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextModelProbe {
    pub ready: bool,
    pub base_url: String,
    pub summary: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeBriefRequest {
    pub brief: String,
    pub template: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativePlan {
    pub template: String,
    pub eyebrow: String,
    pub title: String,
    pub subtitle: String,
    pub cta: String,
    pub image_prompt: String,
    pub negative_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRequest {
    pub template: String,
    pub title: String,
    pub subtitle: String,
    pub cta: String,
    pub eyebrow: String,
    #[serde(default)]
    pub background_image_path: String,
    pub output_name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStatus {
    pub running: bool,
    pub address: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub image_generation: bool,
    pub video_generation: bool,
    pub loras: bool,
    pub svg_composition: bool,
    pub text_design_generation: bool,
    pub automation_api: bool,
    pub engine_mode: EngineMode,
}
