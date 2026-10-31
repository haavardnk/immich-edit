import type { MaskKind, SemanticClass } from '#lib/api/masks.ts';
import { listModels, type ModelKind } from '#lib/api/models.ts';

function isMaskKind(kind: ModelKind): kind is MaskKind {
  return kind !== 'denoise';
}

class MaskModelsState {
  kinds = $state<{ kind: MaskKind; installed: boolean }[]>([]);
  semanticClasses = $state<SemanticClass[]>([]);
  enabled = $state(false);
  failed = $state(false);

  get unavailable(): string | null {
    if (this.failed) return 'Could not reach the server to check for AI models.';
    if (!this.enabled)
      return 'AI masks are turned off on this server. An administrator can enable the segmentation runtime.';
    if (this.kinds.length === 0) return 'No AI mask models are available on this server.';
    return null;
  }

  installed(kind: MaskKind): boolean {
    return this.kinds.some((entry) => entry.kind === kind && entry.installed);
  }

  load = async (): Promise<void> => {
    try {
      const models = await listModels();
      this.failed = false;
      this.enabled = models.enabled;
      this.semanticClasses = models.semantic_classes ?? [];
      this.kinds = models.enabled
        ? [...new Set(models.models.map((m) => m.kind).filter(isMaskKind))].map((kind) => ({
            kind,
            installed: models.models.some((m) => m.kind === kind && m.installed)
          }))
        : [];
    } catch {
      this.failed = true;
      this.enabled = false;
      this.kinds = [];
      this.semanticClasses = [];
    }
  };
}

export const maskModels = new MaskModelsState();
