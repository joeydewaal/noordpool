import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import type { Game } from "$lib/api/types";

const { mockAuth, mockState, scoreMutate, addEventMutate } = vi.hoisted(
  () => {
    const mockAuth = {
      isAdmin: false,
      isModerator: false,
      teamId: null as string | null,
    };
    const mockState = {
      gameData: null as Game | null,
      playersData: [] as unknown[],
    };
    const scoreMutate = vi.fn();
    const addEventMutate = vi.fn();
    return { mockAuth, mockState, scoreMutate, addEventMutate };
  },
);

vi.mock("$app/state", () => ({
  page: { params: { id: "game-1" } },
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: mockAuth,
}));

vi.mock("$lib/api/games", () => ({
  getGame: vi.fn(),
  pollLive: vi.fn().mockResolvedValue({ body: null, etag: null }),
  adjustScore: vi.fn(),
}));

vi.mock("$lib/api/events", () => ({
  createGameEvent: vi.fn(),
  deleteGameEvent: vi.fn(),
}));

vi.mock("$lib/api/players", () => ({
  getPlayers: vi.fn(),
}));

vi.mock("$lib/visibility-polling.svelte", () => ({
  startVisibilityPolling: vi.fn(() => () => {}),
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: (optsFn: () => { queryKey: unknown[] }) => {
    const opts = optsFn();
    const isPlayers = opts.queryKey.includes("players");
    return {
      get data() {
        return isPlayers ? mockState.playersData : mockState.gameData;
      },
      get isPending() {
        return false;
      },
      get isError() {
        return false;
      },
    };
  },
  createMutation: (_optsFn: () => { mutationFn: unknown }) => {
    // The page declares scoreMutation first, then
    // addEventMutation, then deleteEventMutation. We expose
    // the first two by call order so tests can assert on them.
    const calls = (createMutation as unknown as { _n?: number })._n ?? 0;
    (createMutation as unknown as { _n?: number })._n = calls + 1;
    if (calls === 0) {
      return {
        mutate: scoreMutate,
        get isPending() {
          return false;
        },
      };
    }
    if (calls === 1) {
      return {
        mutate: addEventMutate,
        get isPending() {
          return false;
        },
      };
    }
    return {
      mutate: vi.fn(),
      get isPending() {
        return false;
      },
    };
  },
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

// Re-imported above; alias for the call-order trick.
import { createMutation } from "@tanstack/svelte-query";
import Page from "./+page.svelte";

function makeGame(overrides: Partial<Game> = {}): Game {
  return {
    id: "game-1",
    homeTeamId: "team-home",
    homeTeam: { id: "team-home", name: "Noordpool" },
    awayTeamId: "team-away",
    awayTeam: { id: "team-away", name: "PSV" },
    location: "Stadium",
    dateTime: new Date().toISOString(),
    cancelled: false,
    homeScore: 0,
    awayScore: 0,
    version: 0,
    updatedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    status: "live",
    events: [],
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockAuth.isAdmin = false;
  mockAuth.isModerator = false;
  mockAuth.teamId = null;
  mockState.gameData = null;
  mockState.playersData = [];
  (createMutation as unknown as { _n?: number })._n = 0;
});

describe("game detail — score adjuster", () => {
  it("does not render the score adjuster for a non-moderator", () => {
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(
      screen.queryByLabelText("Score adjuster"),
    ).not.toBeInTheDocument();
  });

  it("does not render any score adjuster when the game is not live", () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "scheduled" });
    render(Page);
    expect(
      screen.queryByLabelText("Score adjuster"),
    ).not.toBeInTheDocument();
  });

  it("shows opponent panel when user belongs to a team in this game", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    render(Page);

    expect(screen.getByLabelText("Score adjuster")).toBeInTheDocument();
    expect(screen.getByLabelText("Doelpunt tegenstander")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Doelpunt tegenstander intrekken"),
    ).toBeInTheDocument();
  });

  it("shows the opponent score (away when we are home)", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({
      status: "live",
      homeScore: 3,
      awayScore: 1,
    });
    render(Page);

    const panel = screen.getByLabelText("Score adjuster");
    // 1 = away score = opponent score (we are home)
    expect(panel.textContent).toContain("1");
    // PSV name is the opponent label
    expect(panel.textContent).toContain("PSV");
  });

  it("shows the opponent score (home when we are away)", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-away";
    mockState.gameData = makeGame({
      status: "live",
      homeScore: 2,
      awayScore: 4,
    });
    render(Page);

    const panel = screen.getByLabelText("Score adjuster");
    // We are away, so opponent's score is the home column = 2
    expect(panel.textContent).toContain("2");
  });

  it("shows both-side controls when user is not on either team", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = null;
    mockState.gameData = makeGame({ status: "live" });
    render(Page);

    expect(screen.getByLabelText("Score adjuster")).toBeInTheDocument();
    expect(screen.getByText("Noordpool")).toBeInTheDocument();
    expect(screen.getByText("PSV")).toBeInTheDocument();
  });

  it("opponent + button mutates with side and delta", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    render(Page);

    await fireEvent.click(screen.getByLabelText("Doelpunt tegenstander"));
    expect(scoreMutate).toHaveBeenCalledWith({ side: "away", delta: 1 });
  });

  it("opponent - button mutates with delta = -1", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({
      status: "live",
      awayScore: 2,
    });
    render(Page);

    await fireEvent.click(
      screen.getByLabelText("Doelpunt tegenstander intrekken"),
    );
    expect(scoreMutate).toHaveBeenCalledWith({ side: "away", delta: -1 });
  });

  it("opponent - button is disabled at score 0", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({
      status: "live",
      awayScore: 0,
    });
    render(Page);

    const minusBtn = screen.getByLabelText(
      "Doelpunt tegenstander intrekken",
    ) as HTMLButtonElement;
    expect(minusBtn.disabled).toBe(true);
  });
});

describe("game detail — event form drives goal score", () => {
  it("renders the event form with a Doelpunt option for moderators", () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [
      {
        id: "p1",
        firstName: "Jan",
        lastName: "de Boer",
        shirtNumber: 9,
        active: true,
      },
    ];
    render(Page);

    expect(
      screen.getByText(/registreer je hieronder als gebeurtenis/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Doelpunt" }),
    ).toBeInTheDocument();
  });

  it("submitting a goal event calls the addEvent mutation", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [
      {
        id: "p1",
        firstName: "Jan",
        lastName: "de Boer",
        shirtNumber: 9,
        active: true,
      },
    ];
    render(Page);

    const selects = screen.getAllByRole("combobox") as HTMLSelectElement[];
    await fireEvent.change(selects[0], { target: { value: "p1" } });

    await fireEvent.click(screen.getByRole("button", { name: "Toevoegen" }));

    await waitFor(() => {
      expect(addEventMutate).toHaveBeenCalled();
    });
    const [arg] = addEventMutate.mock.calls[0];
    expect(arg).toMatchObject({
      playerId: "p1",
      eventType: "goal",
    });
  });
});

describe("game detail — live overlay status display", () => {
  it("renders the bezig (live) badge", () => {
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(screen.getByText("LIVE")).toBeInTheDocument();
  });
});
