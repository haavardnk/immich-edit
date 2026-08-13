import { listMaskModels, type MaskKind, type SemanticClass } from '$lib/api/masks';

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

  get clickInstalled(): boolean {
    return this.kinds.some((entry) => entry.kind === 'click' && entry.installed);
  }

  load = async (): Promise<void> => {
    try {
      const models = await listMaskModels();
      this.failed = false;
      this.enabled = models.enabled;
      this.semanticClasses = models.semantic_classes ?? [];
      this.kinds = models.enabled
        ? [...new Set(models.models.map((m) => m.kind))].map((kind) => ({
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
