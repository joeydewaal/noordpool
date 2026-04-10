import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

const { mockAuth, mockQueryState, mockMutate } = vi.hoisted(() => {
  const mockAuth = {
    isAdmin: false,
    isModerator: false,
    playerId: null as string | null,
  };
  const mockQueryState = {
    data: null as any,
    pending: false,
  };
  const mockMutate = vi.fn();
  return { mockAuth, mockQueryState, mockMutate };
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

vi.mock("$lib/api/users", () => ({
  updateUser: vi.fn(),
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: () => ({
    get data() {
      return mockQueryState.data;
    },
    get isPending() {
      return mockQueryState.pending;
    },
    get isError() {
      return false;
    },
  }),
  createMutation: (optsFn: any) => {
    const opts = optsFn ? optsFn() : {};
    return {
      mutate: (vars: any) => {
        mockMutate(vars);
        opts.mutationFn?.(vars);
        opts.onSuccess?.();
      },
      get isPending() {
        return false;
      },
    };
  },
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

import Page from "./+page.svelte";

const linkedUser = {
  id: "user-1",
  email: "jan@example.com",
  firstName: "Jan",
  lastName: "de Boer",
  avatarUrl: null,
  playerId: "test-player-id",
  isAdmin: false,
  isModerator: false,
  roles: ["player"] as const,
};

const testPlayer = {
  id: "test-player-id",
  userId: null,
  firstName: "Jan",
  lastName: "de Boer",
  shirtNumber: 10,
  position: "Centrale middenvelder" as const,
  active: true,
};

describe("Player edit page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAuth.isAdmin = false;
    mockAuth.isModerator = false;
    mockAuth.playerId = null;
    mockQueryState.data = testPlayer;
    mockQueryState.pending = false;
  });

  it("shows all fields for admin", () => {
    mockAuth.isAdmin = true;

    render(Page);

    expect(screen.getByLabelText("Voornaam")).toBeInTheDocument();
    expect(screen.getByLabelText("Achternaam")).toBeInTheDocument();
    expect(screen.getByLabelText("Rugnummer")).toBeInTheDocument();
    expect(screen.getByLabelText("Positie")).toBeInTheDocument();
    expect(screen.getByLabelText("Actief")).toBeInTheDocument();
  });

  it("shows access denied for regular player", () => {
    mockAuth.playerId = "test-player-id";

    render(Page);

    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
    expect(screen.queryByLabelText("Rugnummer")).not.toBeInTheDocument();
  });

  it("shows all fields for moderator", () => {
    mockAuth.isModerator = true;

    render(Page);

    expect(screen.getByLabelText("Voornaam")).toBeInTheDocument();
    expect(screen.getByLabelText("Rugnummer")).toBeInTheDocument();
  });

  it("hides moderator toggle for non-admins", () => {
    mockAuth.isModerator = true;
    mockQueryState.data = { ...testPlayer, userId: "user-1", user: linkedUser };

    render(Page);

    expect(screen.queryByLabelText(/promote jan/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/demote jan/i)).not.toBeInTheDocument();
  });

  it("hides moderator toggle when player has no linked user", () => {
    mockAuth.isAdmin = true;

    render(Page);

    expect(screen.queryByLabelText(/promote jan/i)).not.toBeInTheDocument();
  });

  it("shows promote button for admin viewing linked non-moderator user", () => {
    mockAuth.isAdmin = true;
    mockQueryState.data = { ...testPlayer, userId: "user-1", user: linkedUser };

    render(Page);

    expect(screen.getByLabelText(/promote jan/i)).toBeInTheDocument();
  });

  it("clicking promote calls mutation with isModerator=true", async () => {
    mockAuth.isAdmin = true;
    mockQueryState.data = { ...testPlayer, userId: "user-1", user: linkedUser };

    render(Page);
    await fireEvent.click(screen.getByLabelText(/promote jan/i));

    expect(mockMutate).toHaveBeenCalledWith({
      userId: "user-1",
      isModerator: true,
    });
  });

  it("shows demote button when linked user is already moderator", () => {
    mockAuth.isAdmin = true;
    mockQueryState.data = {
      ...testPlayer,
      userId: "user-1",
      user: {
        ...linkedUser,
        isModerator: true,
        roles: ["player", "moderator"],
      },
    };

    render(Page);

    expect(screen.getByLabelText(/demote jan/i)).toBeInTheDocument();
  });

  it("clicking demote calls mutation with isModerator=false", async () => {
    mockAuth.isAdmin = true;
    mockQueryState.data = {
      ...testPlayer,
      userId: "user-1",
      user: {
        ...linkedUser,
        isModerator: true,
        roles: ["player", "moderator"],
      },
    };

    render(Page);
    await fireEvent.click(screen.getByLabelText(/demote jan/i));

    expect(mockMutate).toHaveBeenCalledWith({
      userId: "user-1",
      isModerator: false,
    });
  });

  it("back button calls history.back so users return to where they came from", async () => {
    mockAuth.isAdmin = true;
    const backSpy = vi
      .spyOn(window.history, "back")
      .mockImplementation(() => {});

    render(Page);
    await fireEvent.click(
      screen.getByRole("button", { name: /terug naar speler/i }),
    );

    expect(backSpy).toHaveBeenCalled();
    backSpy.mockRestore();
  });

  it("saving the form calls history.back on success", async () => {
    mockAuth.isAdmin = true;
    const backSpy = vi
      .spyOn(window.history, "back")
      .mockImplementation(() => {});

    render(Page);
    await fireEvent.click(
      screen.getByRole("button", { name: /wijzigingen opslaan/i }),
    );

    expect(backSpy).toHaveBeenCalled();
    backSpy.mockRestore();
  });

  it("hides moderator toggle when linked user is admin", () => {
    mockAuth.isAdmin = true;
    mockQueryState.data = {
      ...testPlayer,
      userId: "user-1",
      user: { ...linkedUser, isAdmin: true, roles: ["player", "admin"] },
    };

    render(Page);

    expect(screen.queryByLabelText(/promote jan/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/demote jan/i)).not.toBeInTheDocument();
  });
});
