import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import type { Game } from "$lib/api/types";

const { mockAuth, mockState, addEventMutate, mockInvalidateQueries } =
  vi.hoisted(() => {
    const mockAuth = {
      isAdmin: false,
      isModerator: false,
      teamId: null as string | null,
    };
    const mockState = {
      gameData: null as Game | null,
      playersData: [] as unknown[],
      lineupData: null as unknown,
    };
    const addEventMutate = vi.fn();
    const mockInvalidateQueries = vi.fn();
    return { mockAuth, mockState, addEventMutate, mockInvalidateQueries };
  });

vi.mock("$app/state", () => ({
  page: { params: { id: "game-1" } },
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: mockAuth,
}));

vi.mock("$lib/api/games", () => ({
  getGame: vi.fn(),
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
    const isLineup = opts.queryKey.includes("lineup");
    return {
      get data() {
        if (isPlayers) return mockState.playersData;
        if (isLineup) return mockState.lineupData;
        return mockState.gameData;
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
      mutate: (vars: any, callbacks?: any) => {
        addEventMutate(vars, callbacks);
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

function makePlayer(id: string, name: string, teamId: string, shirtNumber = 9) {
  return {
    id,
    firstName: name,
    lastName: "",
    shirtNumber,
    active: true,
    userId: null,
    position: "Spits" as const,
    teamId,
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
  mockState.lineupData = null;
});

describe("game detail — cache invalidation", () => {
  it("invalidates the game cache after adding an event", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [makePlayer("p1", "Jan", "team-home")];

    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));
    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["games", "game-1"],
    });
  });

  it("invalidates the game cache after deleting an event", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({
      status: "live",
      events: [
        {
          id: "evt-1",
          gameId: "game-1",
          playerId: "p1",
          minute: 45,
          eventType: { type: "goal" as const },
          player: {
            id: "p1",
            firstName: "Jan",
            lastName: "",
            shirtNumber: 9,
            active: true,
            position: "Spits" as const,
            teamId: "team-home",
            userId: null,
          },
        },
      ],
    });

    render(Page);
    await fireEvent.click(screen.getByTitle("Verwijderen"));

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["games", "game-1"],
    });
  });
});

describe("game detail — command panel visibility", () => {
  it("does not render the command toggle for a non-moderator", () => {
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(
      screen.queryByRole("button", { name: /wedstrijdbeheer/i }),
    ).not.toBeInTheDocument();
  });

  it("does not render the command toggle when the game is not live", () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "scheduled" });
    render(Page);
    expect(
      screen.queryByRole("button", { name: /wedstrijdbeheer/i }),
    ).not.toBeInTheDocument();
  });

  it("command panel is hidden by default and shown after toggle", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(screen.getByLabelText("Wedstrijdbeheer")).not.toBeVisible();
    await openCommands();
    expect(screen.getByLabelText("Wedstrijdbeheer")).toBeVisible();
  });
});

describe("game detail — lineup-restricted player picker", () => {
  it("shows only the players returned by the backend (lineup-filtered home + all away)", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    // Backend returns pre-filtered list: only Jan from home (lineup), Klaas from away (all)
    mockState.playersData = [
      makePlayer("p1", "Jan", "team-home", 9),
      makePlayer("p3", "Klaas", "team-away", 7),
    ];
    render(Page);
    await openCommands();

    expect(screen.getByRole("button", { name: /jan/i })).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /piet/i }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /klaas/i })).toBeInTheDocument();
  });
});

describe("game detail — player picker (step 1)", () => {
  it("shows home and away team sections after opening commands", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [
      makePlayer("p1", "Jan", "team-home"),
      makePlayer("p2", "Piet", "team-away"),
    ];
    render(Page);
    await openCommands();

    // Both player buttons are visible (one per team section)
    expect(screen.getByRole("button", { name: /jan/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /piet/i })).toBeInTheDocument();
  });

  it("opponent player appears in the away section", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [makePlayer("p2", "Ronaldo", "team-away", 7)];
    render(Page);
    await openCommands();

    expect(
      screen.getByRole("button", { name: /ronaldo/i }),
    ).toBeInTheDocument();
  });

  it("action buttons are not shown before a player is selected", async () => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [makePlayer("p1", "Jan", "team-home")];
    render(Page);
    await openCommands();

    expect(
      screen.queryByRole("button", { name: /doelpunt$/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /toevoegen/i }),
    ).not.toBeInTheDocument();
  });
});

describe("game detail — action picker (step 2)", () => {
  beforeEach(() => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [makePlayer("p1", "Jan", "team-home")];
  });

  async function selectJan() {
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));
  }

  it("shows action buttons and player name after selecting a player", async () => {
    render(Page);
    await selectJan();

    expect(screen.getByText("Jan")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /terug/i })).toBeInTheDocument();
    expect(
      screen.getAllByRole("button", { name: /doelpunt/i }).length,
    ).toBeGreaterThan(0);
    expect(
      screen.getByRole("button", { name: /eigen doelpunt/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /gele kaart/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /rode kaart/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /toevoegen/i }),
    ).toBeInTheDocument();
    // Assist is not a direct action; it appears only after a goal is added.
    expect(
      screen.queryByRole("button", { name: /^assist$/i }),
    ).not.toBeInTheDocument();
  });

  it("back button returns to the player picker", async () => {
    render(Page);
    await selectJan();

    await fireEvent.click(screen.getByRole("button", { name: /terug/i }));

    expect(
      screen.queryByRole("button", { name: /doelpunt$/i }),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /jan/i })).toBeInTheDocument();
  });

  it("submitting calls addEvent with the correct payload", async () => {
    render(Page);
    await selectJan();

    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() => {
      expect(addEventMutate).toHaveBeenCalled();
    });
    const [arg] = addEventMutate.mock.calls[0];
    expect(arg).toMatchObject({ playerId: "p1", eventType: { type: "goal" } });
  });

  it("selecting a different action updates the submitted eventType", async () => {
    render(Page);
    await selectJan();

    await fireEvent.click(screen.getByRole("button", { name: /gele kaart/i }));
    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() => {
      expect(addEventMutate).toHaveBeenCalled();
    });
    const [arg] = addEventMutate.mock.calls[0];
    expect(arg).toMatchObject({
      playerId: "p1",
      eventType: { type: "yellow_card" },
    });
  });
});

describe("game detail — full event emission flow", () => {
  beforeEach(() => {
    mockAuth.isModerator = true;
    mockState.gameData = makeGame({ status: "live" });
    mockState.playersData = [
      makePlayer("p1", "Jan", "team-home", 9),
      makePlayer("p2", "Piet", "team-away", 7),
    ];
  });

  it("emits a goal for a home player with the correct payload", async () => {
    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));
    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() => expect(addEventMutate).toHaveBeenCalledOnce());
    expect(addEventMutate.mock.calls[0][0]).toMatchObject({
      playerId: "p1",
      eventType: { type: "goal" },
    });
  });

  it("emits an own_goal for an away player with the correct payload", async () => {
    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /piet/i }));
    await fireEvent.click(
      screen.getByRole("button", { name: /eigen doelpunt/i }),
    );
    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() => expect(addEventMutate).toHaveBeenCalledOnce());
    expect(addEventMutate.mock.calls[0][0]).toMatchObject({
      playerId: "p2",
      eventType: { type: "own_goal" },
    });
  });

  it("includes the edited minute in the emitted payload", async () => {
    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));

    const minuteInput = screen.getByLabelText(/minuut/i);
    await fireEvent.input(minuteInput, { target: { value: "67" } });

    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() => expect(addEventMutate).toHaveBeenCalledOnce());
    expect(addEventMutate.mock.calls[0][0]).toMatchObject({
      playerId: "p1",
      minute: 67,
    });
  });

  it("closes the dialog after a non-goal submission", async () => {
    addEventMutate.mockImplementation(
      (_data: unknown, callbacks?: { onSuccess?: (r?: unknown) => void }) => {
        callbacks?.onSuccess?.();
      },
    );

    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));
    await fireEvent.click(screen.getByRole("button", { name: /gele kaart/i }));

    expect(screen.getByRole("button", { name: /terug/i })).toBeInTheDocument();

    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    await waitFor(() =>
      expect(screen.getByLabelText("Wedstrijdbeheer")).not.toBeVisible(),
    );
  });

  it("shows assist picker after a goal and closes on 'geen assist'", async () => {
    addEventMutate.mockImplementation(
      (_data: unknown, callbacks?: { onSuccess?: (r?: unknown) => void }) => {
        callbacks?.onSuccess?.({ id: "evt-1", minute: 45 });
      },
    );

    render(Page);
    await openCommands();
    await fireEvent.click(screen.getByRole("button", { name: /jan/i }));
    await fireEvent.click(screen.getByRole("button", { name: /toevoegen/i }));

    // Assist step is shown
    await waitFor(() =>
      expect(screen.getByText(/wie assisteerde/i)).toBeInTheDocument(),
    );

    await fireEvent.click(screen.getByRole("button", { name: /geen assist/i }));

    await waitFor(() =>
      expect(screen.getByLabelText("Wedstrijdbeheer")).not.toBeVisible(),
    );
  });
});

describe("game detail — live overlay status display", () => {
  it("renders the LIVE badge", () => {
    mockState.gameData = makeGame({ status: "live" });
    render(Page);
    expect(screen.getByText("LIVE")).toBeInTheDocument();
  });
});
