import { sampleWhiteBalance, type WhiteBalanceSample } from '#lib/api/edits.ts';
import type { Edits, WhiteBalance } from '#lib/types/edits.ts';
import { errorMessage } from '#lib/utils/errors.ts';
import { SingleFlight } from '#lib/utils/singleFlight.ts';

export type WhiteBalanceReading = { wb: WhiteBalance; error: null } | { wb: null; error: string };

export interface ProbeTarget {
  assetId: string;
  edits: Edits;
}

interface ProbeArgs {
  sample: WhiteBalanceSample;
  epoch: number;
}

export class WhiteBalanceProbe {
  reading = $state<WhiteBalanceReading | null>(null);
  private epoch = 0;
  private flight: SingleFlight<ProbeArgs, WhiteBalanceReading | null>;

  constructor(target: () => ProbeTarget | null) {
    this.flight = new SingleFlight(
      async ({ sample }) => {
        const t = target();
        if (!t) return null;
        try {
          const wb = await sampleWhiteBalance(t.assetId, sample, t.edits, { silent: true });
          return { wb, error: null };
        } catch (e) {
          return { wb: null, error: errorMessage(e) };
        }
      },
      ({ epoch }, reading) => {
        if (epoch === this.epoch) this.reading = reading;
      }
    );
  }

  probe(sample: WhiteBalanceSample): void {
    this.flight.submit({ sample, epoch: this.epoch });
  }

  clear(): void {
    this.epoch += 1;
    this.flight.cancel();
    this.reading = null;
  }
}
