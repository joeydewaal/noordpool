import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import type { Game } from "$lib/api/types";

const { mockAuth, mockState, updateMutate, mockInvalidateQueries } = vi.hoisted(
  () => {
    const mockAuth = { isAdmin: false, isModerator: false };
    const mockState = {
      gameData: null as Game | null,
      teamsData: [
        { id: "team-home", name: "Noordpool" },
        { id: "team-away", name: "Ajax" },
      ],
    };
    const updateMutate = vi.fn();
    const mockInvalidateQueries = vi.fn();
    return { mockAuth, mockState, updateMutate, mockInvalidateQueries };
  },
);

vi.mock("$app/state", () => ({
  page: { params: { id: "game-1" } },
}));

vi.mock("$app/navigation", () => ({
  goto: vi.fn(),
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: mockAuth,
}));

vi.mock("$lib/api/games", () => ({
  getGame: vi.fn(),
  updateGame: vi.fn(),
}));

vi.mock("$lib/api/teams", () => ({
  listTeams: vi.fn(),
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: (optsFn: () => { queryKey: unknown[] }) => {
    const opts = optsFn();
    const isTeams = opts.queryKey.includes("teams");
    return {
      get data() {
        return isTeams ? mockState.teamsData : mockState.gameData;
      },
      get isPending() {
        return false;
      },
      get isError() {
        return false;
      },
    };
  },
  createMutation: (optsFn: any) => {
    const opts = optsFn ? optsFn() : {};
    return {
      mutate: (vars: any) => {
        updateMutate(vars);
        opts.onSuccess?.({});
      },
      get isPending() {
        return false;
      },
    };
  },
  useQueryClient: () => ({
    invalidateQueries: mockInvalidateQueries,
  }),
}));

import Page from "./+page.svelte";

function makeGame(overrides: Partial<Game> = {}): Game {
  return {
    id: "game-1",
    homeTeamId: "team-home",
    homeTeam: { id: "team-home", name: "Noordpool" },
    awayTeamId: "team-away",
    awayTeam: { id: "team-away", name: "Ajax" },
    location: "Stadium",
    dateTime: "2026-06-15T18:00:00Z",
    cancelled: false,
    homeScore: 0,
    awayScore: 0,
    version: 0,
    updatedAt: "2026-06-01T00:00:00Z",
    createdAt: "2026-06-01T00:00:00Z",
    status: "scheduled",
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockAuth.isAdmin = false;
  mockAuth.isModerator = false;
  mockState.gameData = null;
});

describe("Game edit page — cache invalidation", () => {
  it("invalidates the games cache after saving the edit form", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame();

    render(Page);

    const saveButton = await screen.findByRole("button", { name: /opslaan/i });
    await fireEvent.click(saveButton);

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["games"],
    });
  });
});

describe("Game edit page", () => {
  it("shows access denied for regular player", () => {
    mockState.gameData = makeGame();
    render(Page);
    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
  });

  it("populates the datetime-local input with a local-time value (no Z suffix)", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ dateTime: "2026-06-15T18:00:00Z" });

    render(Page);

    const input = (await screen.findByLabelText(
      "Datum & tijd",
    )) as HTMLInputElement;
    expect(input.value).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
    expect(input.value).not.toContain("Z");

    expect(new Date(input.value).toISOString()).toBe(
      "2026-06-15T18:00:00.000Z",
    );
  });

  it("submits dateTime as a UTC ISO string regardless of local TZ", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({
      location: "JC Arena",
      dateTime: "2026-06-15T18:00:00Z",
    });

    render(Page);

    const form = await screen.findByRole("button", { name: /opslaan/i });
    await fireEvent.click(form);

    await waitFor(() => expect(updateMutate).toHaveBeenCalled());
    const [payload] = updateMutate.mock.calls[0];
    expect(payload.homeTeamId).toBe("team-home");
    expect(payload.awayTeamId).toBe("team-away");
    expect(payload.location).toBe("JC Arena");
    expect(payload.dateTime).toMatch(/Z$/);
    expect(new Date(payload.dateTime).toISOString()).toBe(
      "2026-06-15T18:00:00.000Z",
    );
  });

  it("preserves a user-edited datetime through the round-trip", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ dateTime: "2026-06-15T18:00:00Z" });

    render(Page);

    const input = (await screen.findByLabelText(
      "Datum & tijd",
    )) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "2026-07-01T20:30" } });

    await fireEvent.click(screen.getByRole("button", { name: /opslaan/i }));

    await waitFor(() => expect(updateMutate).toHaveBeenCalled());
    const [payload] = updateMutate.mock.calls[0];
    expect(payload.dateTime).toBe(new Date("2026-07-01T20:30").toISOString());
  });
});
