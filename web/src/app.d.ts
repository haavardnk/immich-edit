declare global {
  const __APP_VERSION__: string;
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    interface PageState {
      backLayer?: string;
    }
    // interface Platform {}
  }
}

export {};
