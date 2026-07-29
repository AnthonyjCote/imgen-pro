# Imgen Pro

Imgen Pro is a local-first desktop creative studio for image generation, future video generation, and hybrid marketing design. It is being built for offline automation on a 2019 Intel MacBook Pro with an 8 GB Radeon GPU, with Vulkan/MoltenVK as the primary acceleration target.

## What this scaffold includes

- Tauri 2 desktop shell with a Svelte interface
- Familiar prompt-to-image workflow with model, size, steps, seed, and LoRA selections
- Single-worker generation queue to avoid competing GPU jobs
- Mock engine for validating the complete app workflow before loading a model
- Configurable `stable-diffusion.cpp` process adapter
- Persistent config, job history, logs, and local output files
- Hybrid image + text + SVG marketing composer
- Authenticated HTTP automation API bound to `127.0.0.1`
- Engine probe and primary viability diagnostics

Video is represented in the capability model but intentionally remains disabled until image generation is proven stable on the target Mac.

## Development setup

Prerequisites:

- Node.js 22.12 or newer
- Rust stable toolchain
- Tauri prerequisites for macOS

```bash
npm install
npm run tauri dev
```

The app starts in **Mock viability engine** mode. Generate an image immediately to test the queue, persistence, preview loading, and local file output without downloading a model.

## Connecting stable-diffusion.cpp

1. Build or install a working `sd-cli` binary. The target Mac is expected to use a Vulkan build through MoltenVK.
2. Open **Engine** in Imgen Pro.
3. Change the mode to `stable-diffusion.cpp`.
4. Enter the absolute path to `sd-cli`.
5. Configure either a complete model path or component paths for a FLUX-style model.
6. Save settings and run **Probe engine**.
7. Begin with 512×512 or 768×768, one queued job, and a distilled low-step model.

The adapter supports these profile fields:

- Complete model (`-m`)
- Diffusion model (`--diffusion-model`)
- VAE (`--vae`)
- CLIP-L (`--clip_l`)
- T5XXL/text encoder (`--t5xxl`)
- LoRA directory (`--lora-model-dir`)
- Additional model-specific CLI arguments

The exact arguments can evolve without coupling the frontend to the inference implementation.

## Local automation API

Start the API from the Engine screen. It binds only to loopback and requires the generated bearer token.

```bash
curl http://127.0.0.1:4762/health
```

```bash
curl -X POST http://127.0.0.1:4762/v1/generate/image \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A cinematic product poster on a dark studio set",
    "negative_prompt": "blurry, malformed text",
    "model_id": "primary-model",
    "width": 768,
    "height": 768,
    "steps": 4,
    "seed": 42,
    "loras": []
  }'
```

Available routes:

- `GET /health`
- `GET /v1/capabilities`
- `GET /v1/jobs`
- `GET /v1/jobs/{id}`
- `POST /v1/generate/image`
- `POST /v1/designs/render`

## Repository map

```text
src/
  App.svelte              Desktop product interface
  lib/api.ts              Typed Tauri command client
  lib/types.ts            Shared frontend contracts

src-tauri/src/
  automation.rs           Authenticated loopback HTTP API
  commands.rs             Tauri command boundary
  design.rs               Editable SVG marketing renderer
  engines.rs              Mock and stable-diffusion.cpp adapters
  jobs.rs                 Single-worker local generation queue
  models.rs               Persistent domain contracts
  state.rs                App data paths and persistence

docs/
  ARCHITECTURE.md
  PRIMARY_VIABILITY_TESTS.md
```

## Current safety boundaries

- External API binds to `127.0.0.1`, never all interfaces.
- Automation endpoints require a bearer token.
- Asset previews can only read files inside the app data directory.
- The process adapter passes structured arguments directly to the executable and does not invoke a shell.
- Running-process cancellation is not represented as complete until a real child-process supervisor is added.

## Next milestones

1. Run the mock viability checklist.
2. Compile and probe the Vulkan/MoltenVK `sd-cli` build on the target Mac.
3. Test one distilled image model at conservative dimensions.
4. Add model and LoRA registry management instead of editing one primary profile.
5. Add process log streaming, cancellation, and memory telemetry.
6. Add a CLI adapter for the first viable image-to-video model.
7. Add raster export for hybrid SVG designs.
