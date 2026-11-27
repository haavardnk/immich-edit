class MapFullscreen {
  on = $state(false);

  enter(): void {
    this.on = true;
  }

  exit(): void {
    this.on = false;
  }

  toggle(): void {
    this.on = !this.on;
  }
}

export const mapFullscreen = new MapFullscreen();
