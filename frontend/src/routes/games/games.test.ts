import { render, screen } from "@testing-library/svelte";
import { vi, describe, it, expect, beforeEach } from "vitest";
import Page from "./+page.svelte";

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: vi.fn(),
}));

vi.mock("$lib/api/games", () => ({
  getUpcomingGames: vi.fn(),
  getRecentResults: vi.fn(),
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: { isAdmin: false, isModerator: false },
}));

vi.mock("$lib/utils/date", () => ({
  isThisWeek: vi.fn(),
  isToday: vi.fn(),
}));

import { createQuery } from "@tanstack/svelte-query";
import { page } from "$app/state";
import { isThisWeek, isToday } from "$lib/utils/date";

const mockGame = {
  id: "1",
  opponent: "Ajax",
  location: "Sportpark Noord",
  dateTime: "2026-03-27T15:00:00Z",
  homeAway: "home" as const,
  status: "scheduled" as const,
  homeScore: 0,
  awayScore: 0,
};

function pendingQuery() {
  return { isPending: true, isError: false, data: undefined };
}

function successQuery(data: unknown[]) {
  return { isPending: false, isError: false, data };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(isThisWeek).mockReturnValue(false);
  vi.mocked(isToday).mockReturnValue(false);
  // Reset to default URL (upcoming tab)
  (page as { url: URL }).url = new URL("http://localhost/games");
});

describe("games page loading states", () => {
  it("shows loading when both queries are pending", () => {
    vi.mocked(createQuery)
      .mockReturnValueOnce(pendingQuery() as ReturnType<typeof createQuery>)
      .mockReturnValueOnce(pendingQuery() as ReturnType<typeof createQuery>);

    render(Page);

    // Both tabs render (one hidden), each shows loading
    const loadingEls = screen.getAllByText("Laden...");
    expect(loadingEls.length).toBeGreaterThanOrEqual(1);
  });

  it("shows game list when both queries have data", () => {
    vi.mocked(createQuery)
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>)
      .mockReturnValueOnce(
        successQuery([mockGame]) as ReturnType<typeof createQuery>,
      );

    // Navigate to results tab
    (page as { url: URL }).url = new URL("http://localhost/games?tab=results");

    render(Page);

    expect(screen.getByText("vs Ajax")).toBeInTheDocument();
  });

  it("shows empty state when results query returns no data", () => {
    vi.mocked(createQuery)
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>)
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    (page as { url: URL }).url = new URL("http://localhost/games?tab=results");

    render(Page);

    expect(screen.getByText("Nog geen uitslagen.")).toBeInTheDocument();
  });

  it("shows loading (not empty) on results tab when recent query is still pending after back navigation", () => {
    // This simulates the back navigation bug:
    // upcoming query resolved (has data), but recent query cache was wiped and is still pending
    vi.mocked(createQuery)
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>)
      .mockReturnValueOnce(pendingQuery() as ReturnType<typeof createQuery>);

    (page as { url: URL }).url = new URL("http://localhost/games?tab=results");

    render(Page);

    expect(screen.getByText("Laden...")).toBeInTheDocument();
    expect(screen.queryByText("Nog geen uitslagen.")).not.toBeInTheDocument();
  });
});

describe("this week's match highlight", () => {
  it('shows "Deze week" badge when next match is this week', () => {
    vi.mocked(isThisWeek).mockReturnValue(true);

    vi.mocked(createQuery)
      .mockReturnValueOnce(
        successQuery([mockGame]) as ReturnType<typeof createQuery>,
      )
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    render(Page);

    expect(screen.getByText("Deze week")).toBeInTheDocument();
    expect(screen.getByText("vs Ajax")).toBeInTheDocument();
    expect(screen.getByText("Sportpark Noord")).toBeInTheDocument();
  });

  it('shows "Vandaag" badge when match is today', () => {
    vi.mocked(isThisWeek).mockReturnValue(true);
    vi.mocked(isToday).mockReturnValue(true);

    vi.mocked(createQuery)
      .mockReturnValueOnce(
        successQuery([mockGame]) as ReturnType<typeof createQuery>,
      )
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    render(Page);

    expect(screen.getByText("Vandaag")).toBeInTheDocument();
    expect(screen.queryByText("Deze week")).not.toBeInTheDocument();
  });

  it('shows "LIVE" badge with score when match is live', () => {
    vi.mocked(isThisWeek).mockReturnValue(true);

    const liveGame = {
      ...mockGame,
      status: "live" as const,
      homeScore: 2,
      awayScore: 1,
    };

    vi.mocked(createQuery)
      .mockReturnValueOnce(
        successQuery([liveGame]) as ReturnType<typeof createQuery>,
      )
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    render(Page);

    expect(screen.getByText("LIVE")).toBeInTheDocument();
    expect(screen.getByText(/2\s*[–-]\s*1/)).toBeInTheDocument();
  });

  it("shows no highlight when next match is not this week", () => {
    vi.mocked(isThisWeek).mockReturnValue(false);

    vi.mocked(createQuery)
      .mockReturnValueOnce(
        successQuery([mockGame]) as ReturnType<typeof createQuery>,
      )
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    render(Page);

    expect(screen.queryByText("Deze week")).not.toBeInTheDocument();
    expect(screen.queryByText("Vandaag")).not.toBeInTheDocument();
  });

  it("does not duplicate highlighted match in the upcoming list", () => {
    vi.mocked(isThisWeek).mockReturnValue(true);

    const secondGame = {
      ...mockGame,
      id: "2",
      opponent: "Feyenoord",
    };

    vi.mocked(createQuery)
      .mockReturnValueOnce(
        successQuery([mockGame, secondGame]) as ReturnType<typeof createQuery>,
      )
      .mockReturnValueOnce(successQuery([]) as ReturnType<typeof createQuery>);

    render(Page);

    // Ajax appears once in the highlight, not in the regular list
    const ajaxElements = screen.getAllByText("vs Ajax");
    expect(ajaxElements).toHaveLength(1);

    // Feyenoord appears in the regular list
    expect(screen.getByText("vs Feyenoord")).toBeInTheDocument();
  });
});
