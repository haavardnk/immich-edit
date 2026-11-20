export interface PixelFrame {
  pixels: Uint8Array<ArrayBuffer>;
  color_space: PredefinedColorSpace;
  width: number;
  height: number;
}

export type MakeBitmap = (frame: PixelFrame) => Promise<ImageBitmap>;

export const pixelBitmap: MakeBitmap = (frame) => {
  const { buffer, byteOffset, byteLength } = frame.pixels;
  const data = new ImageData(
    new Uint8ClampedArray(buffer, byteOffset, byteLength),
    frame.width,
    frame.height,
    { colorSpace: frame.color_space }
  );
  return createImageBitmap(data);
};

export async function withBitmap(value: unknown, make: MakeBitmap): Promise<unknown> {
  const { pixels, color_space, ...rest } = value as PixelFrame & Record<string, unknown>;
  const bitmap = await make({ ...rest, pixels, color_space } as PixelFrame);
  return { ...rest, bitmap };
}
