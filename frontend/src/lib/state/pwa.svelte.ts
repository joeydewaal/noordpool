class PwaState {
  deferredPrompt: any = $state(null);

  get installable() {
    return this.deferredPrompt !== null;
  }

  async install() {
    if (!this.deferredPrompt) return;

    this.deferredPrompt.prompt();
    const { outcome } = await this.deferredPrompt.userChoice;
    this.deferredPrompt = null;

    return outcome;
  }
}

export const pwa = new PwaState();
