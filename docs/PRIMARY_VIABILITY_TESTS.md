# Primary Viability Tests

These tests deliberately separate application viability from model viability. Do not debug the desktop app, Vulkan, model compatibility, and marketing composition at the same time.

## P0 — Application workflow without inference

### P0.1 Desktop launch

- Run `npm install`.
- Run `npm run tauri dev`.
- Confirm the app opens and the Engine screen reports mock mode.

Pass condition: no blank window, frontend errors, or Rust panic.

### P0.2 Job queue

- Enter a prompt.
- Generate three mock images quickly.
- Confirm one job runs at a time and the others remain queued.

Pass condition: all three jobs complete and each has a unique local output.

### P0.3 Persistence

- Quit the app.
- Reopen it.
- Confirm configuration, job history, and previews return.

Pass condition: jobs and outputs survive restart.

### P0.4 Hybrid composition

- Open **Compose** while the text provider is set to **Mock creative planner**.
- Enter a creative brief and generate copy plus visual direction.
- Open the generated image prompt in **Create**.
- Select a completed mock output.
- Choose **Use in design**.
- Render an editable SVG.
- Open the saved SVG in a browser or vector editor.

Pass condition: the image layer is embedded and the text remains SVG/HTML text rather than baked into the generated image.

### P0.5 Automation

- Start the local API from Engine.
- Call `/health` without authentication.
- Call `/v1/capabilities` with the bearer token.
- Submit a mock image job through `/v1/generate/image`.
- Poll `/v1/jobs/{id}` until complete.

Pass condition: external automation can create and inspect jobs while the app is open.

## P1 — Engine process viability

### P1.1 Engine probe

- Compile a Vulkan/MoltenVK build of `stable-diffusion.cpp`.
- Set the absolute `sd-cli` path.
- Switch engine mode.
- Run **Probe engine**.

Pass condition: the binary launches and help output appears in diagnostics.

### P1.2 Conservative generation

Use one known-compatible distilled or quantized model.

Suggested first envelope:

- 512×512
- 4 steps for a distilled model
- batch size 1
- no LoRA
- fixed seed

Pass condition: a PNG is produced, shown in the gallery, and persists after restart.

### P1.3 Backend stability

Run the same fixed request five times.

Record:

- load time
- generation time
- whether the process crashes
- whether the system becomes unresponsive
- output path and logs

Pass condition: five consecutive jobs finish without requiring an app restart.

## P2 — Model and LoRA viability

### P2.1 Resolution ladder

Test the same prompt at:

1. 512×512
2. 768×768
3. 1024×1024 only if the first two are stable

Stop increasing resolution as soon as memory pressure, severe swapping, or process instability appears.

### P2.2 LoRA

- Add one LoRA profile compatible with the selected model family.
- Generate at strengths 0.5, 0.8, and 1.0.
- Confirm the command log includes the expected LoRA prompt token and directory.

Pass condition: the LoRA changes the output and does not destabilize generation.

## P3 — Automation reliability

- Submit ten jobs through the loopback API.
- Confirm they remain serialized.
- Poll job status and collect output paths.
- Restart the app and verify history remains available.

Pass condition: automation does not bypass queue limits or corrupt state.

## Explicitly deferred

The following should not be treated as complete during primary viability testing:

- running child-process cancellation
- multiple simultaneous GPU workers
- video inference
- model downloading
- LoRA import UI
- canvas-style freeform editing
- SVG-to-PNG raster export
- production code signing and bundling
