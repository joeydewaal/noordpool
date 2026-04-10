import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { startVisibilityPolling } from "./visibility-polling.svelte";

function setVisibility(state: "visible" | "hidden") {
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    get: () => state,
  });
  document.dispatchEvent(new Event("visibilitychange"));
}

describe("startVisibilityPolling", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    setVisibility("visible");
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("runs the tick immediately and again after visibleMs", async () => {
    const tick = vi.fn().mockResolvedValue(undefined);
    const stop = startVisibilityPolling(tick, {
      visibleMs: 1000,
      hiddenMs: 5000,
    });

    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(1000);
    expect(tick).toHaveBeenCalledTimes(2);

    stop();
  });

  it("uses hiddenMs cadence after a tick completes while hidden", async () => {
    const tick = vi.fn().mockResolvedValue(undefined);
    const stop = startVisibilityPolling(tick, {
      visibleMs: 1000,
      hiddenMs: 5000,
    });

    // Tick #1 fires immediately and schedules the next at visibleMs
    // because the tab is still visible.
    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(1);

    setVisibility("hidden");

    // Tick #2 fires at the already-scheduled visibleMs interval, then
    // reschedules at hiddenMs because the tab is now hidden.
    await vi.advanceTimersByTimeAsync(1000);
    expect(tick).toHaveBeenCalledTimes(2);

    // 4999 ms later, still no tick (would have been a visible-cadence
    // tick).
    await vi.advanceTimersByTimeAsync(4999);
    expect(tick).toHaveBeenCalledTimes(2);

    // Hits the hiddenMs boundary.
    await vi.advanceTimersByTimeAsync(1);
    expect(tick).toHaveBeenCalledTimes(3);

    stop();
  });

  it("fires an immediate tick when the tab becomes visible again", async () => {
    const tick = vi.fn().mockResolvedValue(undefined);
    const stop = startVisibilityPolling(tick, {
      visibleMs: 1000,
      hiddenMs: 5000,
    });

    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(1);

    // Drain the visible-cadence tick so the next one is scheduled at
    // hiddenMs.
    setVisibility("hidden");
    await vi.advanceTimersByTimeAsync(1000);
    expect(tick).toHaveBeenCalledTimes(2);

    // Sit hidden for a while — no extra ticks at the visible cadence.
    await vi.advanceTimersByTimeAsync(2000);
    expect(tick).toHaveBeenCalledTimes(2);

    // Switching back to visible cancels the pending hiddenMs timer and
    // fires an immediate tick.
    setVisibility("visible");
    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(3);

    stop();
  });

  it("stops calling tick after stop()", async () => {
    const tick = vi.fn().mockResolvedValue(undefined);
    const stop = startVisibilityPolling(tick, {
      visibleMs: 1000,
      hiddenMs: 5000,
    });

    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(1);
    stop();

    await vi.advanceTimersByTimeAsync(10_000);
    expect(tick).toHaveBeenCalledTimes(1);
  });

  it("swallows errors thrown by tick and keeps polling", async () => {
    const tick = vi
      .fn()
      .mockRejectedValueOnce(new Error("boom"))
      .mockResolvedValue(undefined);
    const stop = startVisibilityPolling(tick, {
      visibleMs: 1000,
      hiddenMs: 5000,
    });

    await vi.advanceTimersByTimeAsync(0);
    expect(tick).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1000);
    expect(tick).toHaveBeenCalledTimes(2);

    stop();
  });
});
