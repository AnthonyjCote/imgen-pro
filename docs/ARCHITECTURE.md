# Architecture

## Product boundary

Imgen Pro owns the workflow, automation, reproducibility, composition, and asset library. Inference engines remain replaceable adapters.

```text
Svelte desktop interface
        │
        ▼
Tauri command boundary
        │
        ├── Persistent app state
        ├── Single-worker job queue
        ├── Hybrid SVG design renderer
        └── Loopback automation API
                │
                ▼
        Inference adapter contract
          ├── Mock viability engine
          ├── stable-diffusion.cpp process
          └── Future video CLI adapter
```

## Why a process adapter first

A child-process boundary isolates the desktop shell from model crashes, Vulkan failures, and memory exhaustion. It also permits separate Vulkan, Metal, and CPU binaries without rebuilding the entire app.

Direct Rust or C bindings can be evaluated only after the target machine proves stable.

## Queue model

The initial queue permits one active generation job. This is intentional for an 8 GB discrete GPU and prevents simultaneous jobs from competing for VRAM and system memory.

Each job persists:

- complete request
- model profile ID
- LoRA selections
- timestamps
- status and progress
- logs
- output metadata
- error message

## Engine profiles

A model profile separates model components from the UI. The process adapter currently maps profile fields to stable-diffusion.cpp arguments and allows extra arguments for model-specific requirements.

Future registry work should add:

- model file hashing
- quantization metadata
- license metadata
- capability detection
- recommended dimensions and steps
- compatibility rules
- multiple named profiles

## Hybrid composition

The first composer is template-driven instead of a general canvas editor.

```text
Generated image asset
       +
Structured headline, body, CTA
       +
SVG layout template and vector shapes
       =
Editable marketing SVG
```

Generated imagery is embedded as a data URI. Typography and geometric elements remain editable and crisp.

## Automation boundary

The HTTP API:

- binds to `127.0.0.1`
- requires a bearer token except for `/health`
- submits work through the same queue as the UI
- returns job IDs rather than blocking for full generation

Future hardening can add token rotation, per-client scopes, request limits, and an option to disable the server completely.

## Video path

Video should use the same normalized job and asset contracts but a separate adapter. On the target Mac, the first video milestone should test short, low-resolution image-to-video generation through a CLI process rather than assuming a full-resolution text-to-video workflow is viable.
