import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

const { mockAuth, mockInvalidateQueries, mockGoto } = vi.hoisted(() => ({
  mockAuth: { isAdmin: false, isModerator: false },
  mockInvalidateQueries: vi.fn(),
  mockGoto: vi.fn(),
}));

const mockTeams = [
  { id: "home-id", name: "Team A" },
  { id: "away-id", name: "Team B" },
];

vi.mock("$app/navigation", () => ({ goto: mockGoto }));

vi.mock("$lib/state/auth.svelte", () => ({ auth: mockAuth }));

vi.mock("$lib/api/games", () => ({ createGame: vi.fn() }));

vi.mock("$lib/api/teams", () => ({
  listTeams: vi.fn(),
  createTeam: vi.fn(),
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: (optsFn: any) => {
    const opts = optsFn();
    const isTeams = opts.queryKey.includes("teams");
    return {
      get data() {
        return isTeams ? mockTeams : null;
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
        opts.mutationFn?.(vars);
        opts.onSuccess?.({});
      },
      get isPending() {
        return false;
      },
    };
  },
  useQueryClient: () => ({ invalidateQueries: mockInvalidateQueries }),
}));

import Page from "./+page.svelte";

describe("New game page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAuth.isAdmin = false;
    mockAuth.isModerator = false;
  });

  it("shows access denied for non-admin/moderator", () => {
    render(Page);
    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
    expect(screen.queryByLabelText("Locatie")).not.toBeInTheDocument();
  });

  it("shows the form for admin", () => {
    mockAuth.isAdmin = true;
    render(Page);
    expect(screen.getByLabelText("Thuisploeg")).toBeInTheDocument();
    expect(screen.getByLabelText("Uitploeg")).toBeInTheDocument();
    expect(screen.getByLabelText("Locatie")).toBeInTheDocument();
    expect(screen.getByLabelText("Datum & tijd")).toBeInTheDocument();
  });

  it("invalidates the games cache after successful creation", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.change(screen.getByLabelText("Thuisploeg"), {
      target: { value: "home-id" },
    });
    await fireEvent.change(screen.getByLabelText("Uitploeg"), {
      target: { value: "away-id" },
    });

    await fireEvent.submit(
      screen
        .getByRole("button", { name: /wedstrijd aanmaken/i })
        .closest("form")!,
    );

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["games"],
    });
  });

  it("navigates to the games list after successful creation", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.change(screen.getByLabelText("Thuisploeg"), {
      target: { value: "home-id" },
    });
    await fireEvent.change(screen.getByLabelText("Uitploeg"), {
      target: { value: "away-id" },
    });

    await fireEvent.submit(
      screen
        .getByRole("button", { name: /wedstrijd aanmaken/i })
        .closest("form")!,
    );

    expect(mockGoto).toHaveBeenCalledWith("/games");
  });

  it("invalidates the teams cache after creating a new team", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.input(screen.getByPlaceholderText("Ploegnaam"), {
      target: { value: "Nieuw Team" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "+ Toevoegen" }));

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["teams"],
    });
  });
});
