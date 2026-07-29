<script lang="ts">
  import type { CreativePlan, DesignRequest, GeneratedAsset } from "../lib/types";

  export let design: DesignRequest;
  export let creativeBrief: string;
  export let creativePlan: CreativePlan | null;
  export let renderedDesign: GeneratedAsset | null;
  export let previews: Record<string, string>;
  export let busy: boolean;
  export let onCreativeBriefChange: (value: string) => void;
  export let onGeneratePlan: () => void;
  export let onOpenImageGenerator: () => void;
  export let onRender: () => void;
</script>

<section class="composer-layout">
  <div class="control-panel">
    <div class="panel-intro">
      <p class="eyebrow">HYBRID DESIGN</p>
      <h2>Image + SVG composer</h2>
      <p>Use a local language model to plan the copy and visual direction, then keep typography and vector elements crisp over a separately generated image.</p>
    </div>

    <div class="ai-brief-card">
      <div class="section-heading">
        <div>
          <span>Local creative director</span>
          <small>Produces editable copy plus a clean image-only prompt.</small>
        </div>
        <span class="count-pill">LLM</span>
      </div>
      <label class="field">
        <span>Creative brief</span>
        <textarea
          rows="5"
          value={creativeBrief}
          oninput={(event) => onCreativeBriefChange((event.currentTarget as HTMLTextAreaElement).value)}
          placeholder="Create a premium web hero for a local service business. Modern, confident, lots of negative space, clear value proposition..."
        ></textarea>
      </label>
      <button class="secondary-button full-width" disabled={busy} onclick={onGeneratePlan}>
        {busy ? "Planning…" : "Generate copy + visual direction"}
      </button>

      {#if creativePlan}
        <div class="creative-plan-preview">
          <span>Image prompt</span>
          <p>{creativePlan.image_prompt}</p>
          <button class="text-button" onclick={onOpenImageGenerator}>Open in image generator →</button>
        </div>
      {/if}
    </div>

    <label class="field">
      <span>Template</span>
      <select bind:value={design.template}>
        <option value="feature-poster">Feature poster</option>
        <option value="web-hero">Web hero</option>
      </select>
    </label>
    <label class="field"><span>Eyebrow</span><input bind:value={design.eyebrow} /></label>
    <label class="field"><span>Headline</span><textarea rows="3" bind:value={design.title}></textarea></label>
    <label class="field"><span>Supporting copy</span><textarea rows="4" bind:value={design.subtitle}></textarea></label>
    <label class="field"><span>Call to action</span><input bind:value={design.cta} /></label>
    <label class="field">
      <span>Generated image path</span>
      <input bind:value={design.background_image_path} placeholder="Choose an output from the Library" />
    </label>
    <div class="field-grid two">
      <label class="field"><span>Width</span><input type="number" bind:value={design.width} /></label>
      <label class="field"><span>Height</span><input type="number" bind:value={design.height} /></label>
    </div>
    <button class="primary-button" disabled={busy} onclick={onRender}>Render editable SVG</button>
  </div>

  <div class="canvas-stage">
    <div class="canvas-toolbar">
      <span>{design.width} × {design.height}</span>
      <span>SVG output</span>
    </div>
    <div class="canvas-frame" style={`aspect-ratio: ${design.width} / ${design.height}`}>
      {#if renderedDesign && previews[renderedDesign.path]}
        <img src={previews[renderedDesign.path]} alt="Rendered marketing design" />
      {:else}
        <div class="design-placeholder">
          <span>{design.eyebrow}</span>
          <h2>{design.title}</h2>
          <p>{design.subtitle}</p>
          <button>{design.cta}</button>
        </div>
      {/if}
    </div>
    {#if renderedDesign}
      <code class="path-readout">{renderedDesign.path}</code>
    {/if}
  </div>
</section>
