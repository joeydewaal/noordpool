import { render, screen } from "@testing-library/svelte";
import { vi, describe, it, expect, beforeEach } from "vitest";
import Header from "./Header.svelte";
import type { Game } from "$lib/api/types";

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: vi.fn(),
}));

vi.mock("$lib/api/games", () => ({
  getUpcomingGames: vi.fn(),
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: {
    isAuthenticated: false,
    isAdmin: false,
    isModerator: false,
    user: null,
  },
}));

vi.mock("$lib/state/pwa.svelte", () => ({
  pwa: { installable: false, install: vi.fn() },
}));

vi.mock("$lib/state/theme.svelte", () => ({
  theme: { isDark: false },
}));

vi.mock("$app/environment", () => ({
  browser: true,
}));

import { createQuery } from "@tanstack/svelte-query";

function mockUpcoming(data: Game[]) {
  vi.mocked(createQuery).mockReturnValue({
    data,
    isPending: false,
    isError: false,
  } as ReturnType<typeof createQuery>);
}

function makeGame(overrides: Partial<Game> = {}): Game {
  return {
    id: "game-1",
    opponent: "Ajax",
    location: "Stadium",
    dateTime: new Date().toISOString(),
    homeAway: "home",
    cancelled: false,
    homeScore: 0,
    awayScore: 0,
    version: 0,
    updatedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    status: "scheduled",
    ...overrides,
  };
}

/** Build a Date for `time` on today's local date. */
function todayAt(hour: number, minute = 0): Date {
  const d = new Date();
  d.setHours(hour, minute, 0, 0);
  return d;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Header today-game indicator", () => {
  it("shows no dot or subtitle when there is no game today", () => {
    const tomorrow = new Date();
    tomorrow.setDate(tomorrow.getDate() + 1);
    mockUpcoming([makeGame({ dateTime: tomorrow.toISOString() })]);

    render(Header);

    expect(
      screen.queryByLabelText("Wedstrijd vandaag"),
    ).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Wedstrijd live")).not.toBeInTheDocument();
    expect(screen.queryByText(/^vs /)).not.toBeInTheDocument();
  });

  it("shows a static red dot for a scheduled game today", () => {
    mockUpcoming([
      makeGame({
        opponent: "PSV",
        dateTime: todayAt(20, 0).toISOString(),
        status: "scheduled",
      }),
    ]);

    render(Header);

    const dots = screen.getAllByLabelText("Wedstrijd vandaag");
    // Both desktop sidebar and mobile bar render the dot.
    expect(dots.length).toBe(2);
    for (const dot of dots) {
      expect(dot.className).not.toContain("animate-pulse");
    }
  });

  it("shows a pulsing red dot for a live game today", () => {
    mockUpcoming([
      makeGame({
        opponent: "Feyenoord",
        dateTime: todayAt(14, 0).toISOString(),
        status: "live",
      }),
    ]);

    render(Header);

    const dots = screen.getAllByLabelText("Wedstrijd live");
    expect(dots.length).toBe(2);
    for (const dot of dots) {
      expect(dot.className).toContain("animate-pulse");
    }
  });

  it('renders the "vs Opponent" subtitle in the desktop sidebar only', () => {
    mockUpcoming([
      makeGame({
        opponent: "Vitesse",
        dateTime: todayAt(15, 0).toISOString(),
        status: "scheduled",
      }),
    ]);

    const { container } = render(Header);

    // One match in the desktop sidebar (.hidden md:flex), zero in mobile.
    const subtitles = container.querySelectorAll(
      "span.text-xs.text-surface-400",
    );
    const matching = Array.from(subtitles).filter((el) =>
      el.textContent?.includes("vs Vitesse"),
    );
    expect(matching.length).toBe(1);
  });

  it("prefers a live game over a scheduled one when both are today", () => {
    mockUpcoming([
      makeGame({
        id: "a",
        opponent: "Early",
        dateTime: todayAt(10, 0).toISOString(),
        status: "scheduled",
      }),
      makeGame({
        id: "b",
        opponent: "NowPlaying",
        dateTime: todayAt(14, 0).toISOString(),
        status: "live",
      }),
    ]);

    render(Header);

    expect(screen.getAllByLabelText("Wedstrijd live").length).toBe(2);
    // Subtitle picks the live game
    expect(screen.getByText("vs NowPlaying")).toBeInTheDocument();
  });

  it("falls back to the earliest game today when none are live", () => {
    mockUpcoming([
      makeGame({
        id: "late",
        opponent: "Late",
        dateTime: todayAt(20, 0).toISOString(),
        status: "scheduled",
      }),
      makeGame({
        id: "early",
        opponent: "Early",
        dateTime: todayAt(12, 0).toISOString(),
        status: "scheduled",
      }),
    ]);

    render(Header);

    expect(screen.getByText("vs Early")).toBeInTheDocument();
  });

  it("ignores games on a different local date", () => {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    yesterday.setHours(20, 0, 0, 0);

    mockUpcoming([makeGame({ dateTime: yesterday.toISOString() })]);

    render(Header);

    expect(screen.queryByLabelText(/Wedstrijd/)).not.toBeInTheDocument();
  });

  it("handles an empty upcoming list", () => {
    mockUpcoming([]);
    render(Header);
    expect(screen.queryByLabelText(/Wedstrijd/)).not.toBeInTheDocument();
  });
});
