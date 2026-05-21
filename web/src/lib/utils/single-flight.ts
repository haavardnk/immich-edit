export class SingleFlight<TArgs, TResult> {
  private current: AbortController | null = null;
  private pending: TArgs | null = null;
  private running = false;
  private fn: (args: TArgs, signal: AbortSignal) => Promise<TResult>;
  private onResult: (args: TArgs, result: TResult) => void;
  private onError: (err: unknown) => void;

  constructor(
    fn: (args: TArgs, signal: AbortSignal) => Promise<TResult>,
    onResult: (args: TArgs, result: TResult) => void,
    onError: (err: unknown) => void = () => {}
  ) {
    this.fn = fn;
    this.onResult = onResult;
    this.onError = onError;
  }

  submit(args: TArgs): void {
    if (this.running) {
      this.pending = args;
      this.current?.abort();
      return;
    }
    void this.run(args);
  }

  private async run(args: TArgs): Promise<void> {
    this.running = true;
    this.current = new AbortController();
    try {
      const result = await this.fn(args, this.current.signal);
      this.onResult(args, result);
    } catch (err) {
      if (!(err instanceof DOMException && err.name === 'AbortError')) {
        this.onError(err);
      }
    } finally {
      this.running = false;
      this.current = null;
      const next = this.pending;
      this.pending = null;
      if (next !== null) {
        void this.run(next);
      }
    }
  }

  cancel(): void {
    this.pending = null;
    this.current?.abort();
  }
}
