import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

const { mockAuth, mockQueryState } = vi.hoisted(() => {
  const mockAuth = {
    isAdmin: false,
    isModerator: false,
    playerId: null as string | null,
  };
  const mockQueryState = {
    playerData: null as any,
    statsData: null as any,
  };
  return { mockAuth, mockQueryState };
});

vi.mock("$app/state", () => ({
  page: { params: { id: "test-player-id" } },
}));

vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: mockAuth,
}));

vi.mock("$lib/api/players", () => ({
  getPlayer: vi.fn(),
  updatePlayer: vi.fn(),
}));

vi.mock("$lib/api/events", () => ({
  getPlayerStats: vi.fn(),
}));

// Mock chart components to avoid layerchart/matchMedia issues
vi.mock("$lib/components/charts/PlayerStatsBar.svelte", () => ({
  default: {},
}));
vi.mock("$lib/components/charts/PlayerTimeline.svelte", () => ({
  default: {},
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: (optsFn: any) => {
    const opts = optsFn();
    const isStats = opts.queryKey.includes("stats");
    return {
      get data() {
        return isStats ? mockQueryState.statsData : mockQueryState.playerData;
      },
      get isPending() {
        return false;
      },
      get isError() {
        return !isStats && !mockQueryState.playerData;
      },
    };
  },
  createMutation: () => ({
    mutate: vi.fn(),
    get isPending() {
      return false;
    },
  }),
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

import Page from "./+page.svelte";

const testPlayer = {
  id: "test-player-id",
  userId: null,
  firstName: "Jan",
  lastName: "de Boer",
  shirtNumber: 10,
  position: "Centrale middenvelder" as const,
  active: true,
};

describe("Player detail page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAuth.isAdmin = false;
    mockAuth.isModerator = false;
    mockAuth.playerId = null;
    mockQueryState.playerData = testPlayer;
    mockQueryState.statsData = null;
  });

  it("hides edit button for regular player viewing own player", () => {
    mockAuth.playerId = "test-player-id";

    render(Page);

    expect(screen.queryByText("Bewerken")).not.toBeInTheDocument();
  });

  it("hides edit button for unauthenticated user", () => {
    render(Page);

    expect(screen.queryByText("Bewerken")).not.toBeInTheDocument();
  });

  it("shows edit and deactivate for admin", () => {
    mockAuth.isAdmin = true;

    render(Page);

    expect(screen.getByText("Bewerken")).toBeInTheDocument();
    expect(screen.getByText("Deactiveren")).toBeInTheDocument();
  });

  it("shows edit and deactivate for moderator", () => {
    mockAuth.isModerator = true;

    render(Page);

    expect(screen.getByText("Bewerken")).toBeInTheDocument();
    expect(screen.getByText("Deactiveren")).toBeInTheDocument();
  });

  it("back button calls history.back so users return to where they came from", async () => {
    const backSpy = vi
      .spyOn(window.history, "back")
      .mockImplementation(() => {});

    render(Page);
    await fireEvent.click(screen.getByRole("button", { name: /terug/i }));

    expect(backSpy).toHaveBeenCalled();
    backSpy.mockRestore();
  });
});
