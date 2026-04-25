import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

const { mockAuth, mockInvalidateQueries } = vi.hoisted(() => ({
  mockAuth: { isAdmin: false },
  mockInvalidateQueries: vi.fn(),
}));

const testUsers = [
  {
    id: "user-1",
    firstName: "Jan",
    lastName: "de Boer",
    email: "jan@dn.nl",
    roles: [],
    isAdmin: false,
    isModerator: false,
    playerId: null,
    avatarUrl: null,
  },
];

vi.mock("$lib/state/auth.svelte", () => ({ auth: mockAuth }));

vi.mock("$lib/api/users", () => ({
  listUsers: vi.fn(),
  updateUser: vi.fn(),
}));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: () => ({
    get data() {
      return testUsers;
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

describe("Admin users page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAuth.isAdmin = false;
  });

  it("shows access denied for non-admin", () => {
    render(Page);
    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /moderator/i }),
    ).not.toBeInTheDocument();
  });

  it("shows the user list for admin", () => {
    mockAuth.isAdmin = true;
    render(Page);
    expect(screen.getByText("Jan de Boer")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Maak moderator" }),
    ).toBeInTheDocument();
  });

  it("invalidates the users cache after toggling moderator status", async () => {
    mockAuth.isAdmin = true;
    render(Page);

    await fireEvent.click(
      screen.getByRole("button", { name: "Maak moderator" }),
    );

    expect(mockInvalidateQueries).toHaveBeenCalledWith({
      queryKey: ["users"],
    });
  });
});
