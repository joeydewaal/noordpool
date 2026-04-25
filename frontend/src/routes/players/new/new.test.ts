import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

const { mockAuth, mockInvalidateQueries, mockGoto } = vi.hoisted(() => ({
  mockAuth: { isAdmin: false, isModerator: false },
  mockInvalidateQueries: vi.fn(),
  mockGoto: vi.fn(),
}));

vi.mock("$app/navigation", () => ({ goto: mockGoto }));

vi.mock("$lib/state/auth.svelte", () => ({ auth: mockAuth }));

vi.mock("$lib/api/players", () => ({ createPlayer: vi.fn() }));

vi.mock("$lib/api/teams", () => ({ listTeams: vi.fn() }));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: () => ({
    get data() {
      return [];
    },
    get isPending() {
      return false;
    },
    get isError() {
      return false;
    },
  }),
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

describe("New player page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAuth.isAdmin = false;
    mockAuth.isModerator = false;
  });

  it("shows access denied for non-admin/moderator", () => {
    render(Page);
    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
    expect(screen.queryByLabelText("Voornaam")).not.toBeInTheDocument();
  });

  it("shows the form for admin", () => {
    mockAuth.isAdmin = true;
    render(Page);
    expect(screen.getByLabelText("Voornaam")).toBeInTheDocument();
    expect(screen.getByLabelText("Achternaam")).toBeInTheDocument();
    expect(screen.getByLabelText("Rugnummer")).toBeInTheDocument();
    expect(screen.getByLabelText("Positie")).toBeInTheDocument();
  });

  it("shows the form for moderator", () => {
    mockAuth.isModerator = true;
    render(Page);
    expect(screen.getByLabelText("Voornaam")).toBeInTheDocument();
  });

  it("invalidates the players cache after successful creation", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.submit(
      screen.getByRole("button", { name: /speler aanmaken/i }).closest("form")!,
    );

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["players"],
    });
  });

  it("navigates to the players list after successful creation", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.submit(
      screen.getByRole("button", { name: /speler aanmaken/i }).closest("form")!,
    );

    expect(mockGoto).toHaveBeenCalledWith("/players");
  });
});
