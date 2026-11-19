export type TargetTool = 'hsl' | 'curves' | 'bw' | 'grading';

class TargetAdjustStore {
  tool = $state<TargetTool | null>(null);
  last = $state<TargetTool>('hsl');

  toggle = (tool: TargetTool): void => {
    this.last = tool;
    this.tool = this.tool === tool ? null : tool;
  };

  arm = (tool: TargetTool): void => {
    this.last = tool;
    this.tool = tool;
  };

  exit = (): void => {
    this.tool = null;
  };
}

export const targetAdjust = new TargetAdjustStore();
