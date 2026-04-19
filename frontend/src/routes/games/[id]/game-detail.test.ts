import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import type { Game } from "$lib/api/types";

const { mockAuth, mockState, scoreMutate, addEventMutate } = vi.hoisted(() => {
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
});

vi.mock("$app/state", () => ({
  page: { params: { id: "game-1" } },
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: mockAuth,
}));

vi.mock("$lib/api/games", () => ({
  getGame: vi.fn(),
  adjustScore: vi.fn(),
}));

vi.mock("$lib/api/events", () => ({
  createGameEvent: vi.fn(),
  deleteGameEvent: vi.fn(),
}));

vi.mock("$lib/api/players", () => ({
  getPlayers: vi.fn(),
}));

vi.mock("$lib/live-match.svelte", () => ({
  startLiveMatchStream: vi.fn(() => () => {}),
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

async function openCommands() {
  const toggle = screen.getByRole("button", { name: /wedstrijdbeheer/i });
  await fireEvent.click(toggle);
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
    expect(screen.queryByLabelText("Score adjuster")).not.toBeInTheDocument();
  });

  it("does not render the command toggle when the game is not live", () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "scheduled" });
    render(Page);
    expect(
      screen.queryByRole("button", { name: /wedstrijdbeheer/i }),
    ).not.toBeInTheDocument();
  });

  it("score adjuster is hidden by default and shown after toggle", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(screen.queryByLabelText("Score adjuster")).not.toBeInTheDocument();
    await openCommands();
    expect(screen.getByLabelText("Score adjuster")).toBeInTheDocument();
  });

  it("shows both-side controls when user belongs to a team in this game", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    await openCommands();

    expect(screen.getByLabelText("Score adjuster")).toBeInTheDocument();
    expect(screen.getByLabelText("Doelpunt tegenstander")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Doelpunt tegenstander intrekken"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Eigen doelpunt")).toBeInTheDocument();
    expect(
      screen.getByLabelText("Eigen doelpunt intrekken"),
    ).toBeInTheDocument();
  });

  it("own-side + button mutates own side", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    await openCommands();

    await fireEvent.click(screen.getByLabelText("Eigen doelpunt"));
    expect(scoreMutate).toHaveBeenCalledWith({ side: "home", delta: 1 });
  });

  it("shows the opponent score (away when we are home)", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({
      status: "live",
      homeScore: 3,
      awayScore: 1,
    });
    render(Page);
    await openCommands();

    const panel = screen.getByLabelText("Score adjuster");
    expect(panel.textContent).toContain("1");
    expect(panel.textContent).toContain("PSV");
  });

  it("shows the opponent score (home when we are away)", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-away";
    mockState.gameData = makeGame({
      status: "live",
      homeScore: 2,
      awayScore: 4,
    });
    render(Page);
    await openCommands();

    const panel = screen.getByLabelText("Score adjuster");
    expect(panel.textContent).toContain("2");
  });

  it("shows both-side controls when user is not on either team", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = null;
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    await openCommands();

    const panel = screen.getByLabelText("Score adjuster");
    expect(panel).toBeInTheDocument();
    expect(panel.textContent).toContain("Noordpool");
    expect(panel.textContent).toContain("PSV");
  });

  it("opponent + button mutates with side and delta", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    await openCommands();

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
    await openCommands();

    await fireEvent.click(
      screen.getByLabelText("Doelpunt tegenstander intrekken"),
    );
    expect(scoreMutate).toHaveBeenCalledWith({ side: "away", delta: -1 });
  });

  it("opponent - button is disabled at score 0", async () => {
    mockAuth.isModerator = true;
    mockAuth.teamId = "team-home";
    mockState.gameData = makeGame({
      status: "live",
      awayScore: 0,
    });
    render(Page);
    await openCommands();

    const minusBtn = screen.getByLabelText(
      "Doelpunt tegenstander intrekken",
    ) as HTMLButtonElement;
    expect(minusBtn.disabled).toBe(true);
  });
});

describe("game detail — event form drives goal score", () => {
  it("renders the event form with a Doelpunt option for moderators", async () => {
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
    await openCommands();

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
    await openCommands();

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
