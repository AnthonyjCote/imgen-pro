# Imgen Pro

Imgen Pro is a local-first desktop creative studio for image generation, future video generation, and hybrid marketing design. It is being built for offline automation on a 2019 Intel MacBook Pro with an 8 GB Radeon GPU, with Vulkan/MoltenVK as the primary image-acceleration target.

The application keeps two local inference responsibilities separate:

- **Image engine:** `stable-diffusion.cpp` / `sd-cli` for normal images, backgrounds, product visuals, graphic elements, and LoRA workflows.
- **Creative planning engine:** a local OpenAI-compatible endpoint such as `llama-server` for headlines, supporting copy, CTA text, layout direction, and clean image-generation prompts.

Either engine can be used independently. SVG composition is optional; image outputs remain normal standalone files.

## Included in this branch

- Tauri 2 desktop shell with a Svelte interface
- Familiar prompt-to-image workflow with model, size, steps, seed, and LoRA selections
- Image-only presets for square, web hero, wide banner, desktop, and portrait assets
- Single-worker generation queue to avoid competing GPU jobs
- Mock image engine for validating the complete application workflow before loading a model
- Mock creative planner for validating text + image workflows before launching a local LLM
- Configurable `stable-diffusion.cpp` process adapter
- Local OpenAI-compatible text/design model adapter
- Creative brief workflow that produces editable copy plus a separate image prompt
- Persistent configuration, job history, logs, local outputs, and SVG designs
- Hybrid image + text + SVG marketing composer
- Authenticated HTTP automation API bound to `127.0.0.1`
- Engine probes and primary viability diagnostics

Video is represented in the capability model but intentionally remains disabled until image generation is proven stable on the target Mac.

## macOS first run

Prerequisites:

- macOS with Xcode Command Line Tools
- Node.js 22.12 or newer
- Rust stable toolchain

After checking out the branch:

```bash
npm run setup:mac
npm run tauri dev
```

The setup command verifies Node, npm, Rust, Cargo, and Xcode Command Line Tools before installing JavaScript dependencies.

The app starts in **Mock viability engine** mode with the **Mock creative planner** enabled. Generate an image immediately to test the queue, persistence, preview loading, local file output, and SVG composition without downloading an image model or launching a local text model.

## Workflow 1: normal image output

Use **Create** for standalone image files such as:

- website backgrounds
- web hero visuals
- texture and lighting elements
- product photography concepts
- social graphics without typography
- abstract design elements
- images that will be edited elsewhere

Choose a preset or enter custom dimensions, select a model and optional LoRAs, then generate. The output can remain a normal image or be sent to the SVG composer later.

## Workflow 2: local creative planning + hybrid design

Use **Compose** when the final asset needs crisp typography or structured marketing copy.

1. Enter a creative brief.
2. Ask the local text model to generate the headline, supporting copy, CTA, layout template, image prompt, and negative prompt.
3. Open the generated visual prompt in **Create** and generate a clean image without embedded typography.
4. Send that image to **Compose**.
5. Render an editable SVG containing the generated image plus crisp text and vector elements.

The text model never needs to render pixels. The image model never needs to produce readable marketing copy.

## Connecting a local text/design model

Imgen Pro defaults to:

```text
http://127.0.0.1:8080/v1
```

A typical llama.cpp server launch is:

```bash
./llama-server \
  -m /absolute/path/to/your-model.gguf \
  -c 4096 \
  --host 127.0.0.1 \
  --port 8080 \
  --alias local-model
```

Then:

1. Open **Engine**.
2. Confirm the provider is **OpenAI-compatible local server**.
3. Keep the base URL at `http://127.0.0.1:8080/v1`.
4. Set the model name to the server alias, such as `local-model`.
5. Save settings.
6. Select **Probe text model**.
7. Open **Compose** and submit a creative brief.

Any local server implementing compatible `/v1/models` and `/v1/chat/completions` routes can be used.

## Connecting stable-diffusion.cpp

1. Build or install a working `sd-cli` binary. The target Mac is expected to use a Vulkan build through MoltenVK.
2. Open **Engine** in Imgen Pro.
3. Change the image mode to `stable-diffusion.cpp`.
4. Enter the absolute path to `sd-cli`.
5. Configure either a complete model path or component paths for a FLUX-style model.
6. Save settings and run **Probe image engine**.
7. Begin with 512×512 or 768×768, one queued job, and a distilled low-step model.

The adapter supports these profile fields:

- Complete model (`-m`)
- Diffusion model (`--diffusion-model`)
- VAE (`--vae`)
- CLIP-L (`--clip_l`)
- T5XXL/text encoder (`--t5xxl`)
- LLM image text encoder (`--llm`)
- LoRA directory (`--lora-model-dir`)
- Backend target (`--backend`, `--params-backend`, and CPU offload presets)
- Additional model-specific CLI arguments

The frontend submits normalized generation requests and does not construct shell commands.

### Local Z-Image smoke test on the Intel Mac

The current known-good local smoke-test setup is:

- `sd-cli`: `/Users/dev/DEV_TOOLS/stable-diffusion.cpp/build-vulkan/bin/sd-cli`
- Diffusion model: `/Users/dev/DEV_TOOLS/MODELS_IMG/z_image_turbo-Q2_K.gguf`
- VAE: `/Users/dev/DEV_TOOLS/MODELS_IMG/vae/ae.sft`
- LLM image text encoder: `/Users/dev/DEV_TOOLS/MODELS_IMG/text_encoders/Qwen3-4B-Instruct-2507-Q4_K_M.gguf`
- Backend target: `Vulkan / MoltenVK` for the first GPU smoke test, or `CPU fallback` if the GPU run fails
- Extra CLI arguments: `--cfg-scale 1.0 --sampling-method euler`

The Vulkan/MoltenVK build sees the Radeon GPU. A 256x256, one-step Z-Image run successfully placed the diffusion model on VRAM and produced a PNG in about 69 seconds. CPU fallback also produced a valid 256x256 PNG in about 46 seconds. Treat 512x512 and higher-step GPU tests as the next stability ladder rather than the first test.

## Local automation API

Start the API from the Engine screen. It binds only to loopback and requires the generated bearer token.

```bash
curl http://127.0.0.1:4762/health
```

Generate a normal image:

```bash
curl -X POST http://127.0.0.1:4762/v1/generate/image \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A restrained abstract background for a premium SaaS website, charcoal geometry, soft volumetric light, no text",
    "negative_prompt": "text, watermark, logo, clutter, low detail",
    "model_id": "primary-model",
    "width": 1024,
    "height": 576,
    "steps": 4,
    "seed": 42,
    "loras": []
  }'
```

Generate a creative plan:

```bash
curl -X POST http://127.0.0.1:4762/v1/creative/plan \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "brief": "Premium web hero for a local automation consultancy. Professional, clear, modern, and conversion focused.",
    "template": "web-hero",
    "width": 1400,
    "height": 900
  }'
```

Available routes:

- `GET /health`
- `GET /v1/capabilities`
- `GET /v1/jobs`
- `GET /v1/jobs/{id}`
- `POST /v1/generate/image`
- `POST /v1/creative/plan`
- `POST /v1/designs/render`

## Repository map

```text
src/
  App.svelte                  Desktop product interface and workflow state
  lib/api.ts                  Typed Tauri command client
  lib/types.ts                Shared frontend contracts
  views/CreateView.svelte     Standalone image generation
  views/ComposeView.svelte    Local creative planning and SVG composition
  views/EngineView.svelte     Image and text engine configuration

src-tauri/src/
  automation.rs               Authenticated loopback HTTP API
  commands.rs                 Tauri command boundary
  design.rs                   Editable SVG marketing renderer
  engines/                    Mock and stable-diffusion.cpp adapters
  jobs.rs                     Single-worker local generation queue
  models.rs                   Persistent domain contracts
  state.rs                    App data paths and persistence
  text_model.rs               OpenAI-compatible local creative planner

scripts/
  setup-macos.sh              macOS prerequisite and dependency check

docs/
  ARCHITECTURE.md
  PRIMARY_VIABILITY_TESTS.md
```

## Current safety boundaries

- External API binds to `127.0.0.1`, never all interfaces.
- Automation endpoints require a bearer token.
- Asset previews can only read files inside the app data directory.
- The image process adapter passes structured arguments directly to the executable and does not invoke a shell.
- Local text requests use a configured loopback-compatible HTTP endpoint.
- Running-process cancellation is not represented as complete until a real child-process supervisor is added.

## Remaining on-device viability work

1. Run the mock viability checklist.
2. Launch and probe the chosen local text model.
3. Compile and probe the Vulkan/MoltenVK `sd-cli` build on the target Mac.
4. Test one distilled image model at conservative dimensions.
5. Record generation time, memory pressure, stability, and LoRA compatibility.
6. Add model and LoRA registry management instead of editing one primary profile.
7. Add process log streaming, running-process cancellation, and memory telemetry.
8. Add raster export for hybrid SVG designs.
9. Add the first viable image-to-video adapter after image generation is stable.
