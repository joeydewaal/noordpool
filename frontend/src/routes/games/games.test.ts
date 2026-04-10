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

import { createQuery } from "@tanstack/svelte-query";
import { page } from "$app/state";

const mockGame = {
  id: "1",
  opponent: "Ajax",
  dateTime: "2026-03-27T15:00:00Z",
  homeAway: "home" as const,
  status: "completed" as const,
  homeScore: 2,
  awayScore: 1,
};

function pendingQuery() {
  return { isPending: true, isError: false, data: undefined };
}

function successQuery(data: unknown[]) {
  return { isPending: false, isError: false, data };
}

beforeEach(() => {
  vi.clearAllMocks();
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
