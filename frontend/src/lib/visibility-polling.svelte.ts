/**
 * Visibility-aware polling driver.
 *
 * Calls `tick()` on a fixed interval that adapts to `document.visibilityState`:
 *  - foreground tab → `visibleMs` (default 3 s)
 *  - hidden tab     → `hiddenMs`  (default 30 s)
 *
 * It also fires an immediate tick whenever the tab becomes visible again so
 * users see fresh state right after switching back. The driver self-stops by
 * calling the returned `stop()` (typically inside `$effect` cleanup).
 *
 * Why this exists: we deploy on AWS Lambda, so SSE/WebSockets are off the
 * table — see `memory/project_deployment_lambda.md`. Cheap, predictable HTTP
 * polling with a back-off when the tab is hidden is the right shape.
 */
export interface VisibilityPollingOptions {
  visibleMs?: number;
  hiddenMs?: number;
}

export function startVisibilityPolling(
  tick: () => void | Promise<void>,
  opts: VisibilityPollingOptions = {},
): () => void {
  const visibleMs = opts.visibleMs ?? 3_000;
  const hiddenMs = opts.hiddenMs ?? 30_000;

  let timer: ReturnType<typeof setTimeout> | null = null;
  let stopped = false;

  const currentInterval = () =>
    typeof document !== "undefined" && document.visibilityState === "hidden"
      ? hiddenMs
      : visibleMs;

  async function loop() {
    if (stopped) return;
    try {
      await tick();
    } catch {
      // swallow — caller decides how to surface errors via reactive state
    }
    if (stopped) return;
    timer = setTimeout(loop, currentInterval());
  }

  function onVisibilityChange() {
    if (stopped) return;
    if (document.visibilityState === "visible") {
      // Cancel pending tick and run one immediately so the user sees
      // fresh data the moment they refocus the tab.
      if (timer) clearTimeout(timer);
      loop();
    }
  }

  // Kick off
  loop();
  if (typeof document !== "undefined") {
    document.addEventListener("visibilitychange", onVisibilityChange);
  }

  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
    if (typeof document !== "undefined") {
      document.removeEventListener("visibilitychange", onVisibilityChange);
    }
  };
}
