import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";

const { mockAuth, mockLineupPending, capturedOnSuccess } = vi.hoisted(() => {
  const mockAuth = { isAdmin: false, isModerator: false };
  let mockLineupPending = false;
  const capturedOnSuccess = { fn: undefined as (() => void) | undefined };
  return { mockAuth, mockLineupPending, capturedOnSuccess };
});

vi.mock("$app/state", () => ({
  page: {
    params: { id: "game-1" },
    url: { searchParams: new URLSearchParams("edit") },
  },
}));

vi.mock("$lib/state/auth.svelte", () => ({ auth: mockAuth }));
vi.mock("$lib/state/toaster", () => ({ toaster: { success: vi.fn() } }));
vi.mock("$lib/api/games", () => ({ getGame: vi.fn() }));
vi.mock("$lib/api/lineup", () => ({ getLineup: vi.fn(), saveLineup: vi.fn() }));
vi.mock("$lib/api/players", () => ({ getPlayers: vi.fn() }));

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: (optsFn: () => { queryKey: unknown[] }) => {
    const opts = optsFn();
    const key = opts.queryKey as string[];
    return {
      get data() {
        return null;
      },
      get isPending() {
        return key.includes("lineup") ? mockLineupPending : false;
      },
      get isError() {
        return false;
      },
    };
  },
  createMutation: (
    optsFn: () => { mutationFn: unknown; onSuccess?: () => void },
  ) => {
    const opts = optsFn();
    capturedOnSuccess.fn = opts.onSuccess;
    return {
      mutate: vi.fn(),
      get isPending() {
        return false;
      },
    };
  },
  useQueryClient: () => ({ invalidateQueries: vi.fn() }),
}));

import LineupPage from "./+page.svelte";

beforeEach(() => {
  vi.clearAllMocks();
  mockAuth.isAdmin = false;
  mockAuth.isModerator = false;
  capturedOnSuccess.fn = undefined;
});

describe("lineup — ?edit auto-mode", () => {
  it("auto-enters edit mode when navigated with ?edit param", async () => {
    mockAuth.isModerator = true;
    render(LineupPage);
    // Formation buttons only appear in edit mode
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "4-4-2" })).toBeInTheDocument(),
    );
  });

  it("does not auto-enter edit mode when user is not a manager", () => {
    mockAuth.isModerator = false;
    mockAuth.isAdmin = false;
    render(LineupPage);
    // No formation buttons — edit mode was not triggered
    expect(
      screen.queryByRole("button", { name: "4-4-2" }),
    ).not.toBeInTheDocument();
  });

  it("does not re-enter edit mode after saving (autoEditDone prevents loop)", async () => {
    mockAuth.isModerator = true;
    render(LineupPage);

    // Wait for auto-edit to trigger
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "4-4-2" })).toBeInTheDocument(),
    );

    // Simulate save mutation success — this sets editMode = false in the component
    capturedOnSuccess.fn?.();

    // Edit mode should exit: formation selector buttons gone
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "4-4-2" }),
      ).not.toBeInTheDocument(),
    );

    // Wait another tick to confirm the $effect does NOT re-enter edit mode
    await new Promise((r) => setTimeout(r, 30));
    expect(
      screen.queryByRole("button", { name: "4-4-2" }),
    ).not.toBeInTheDocument();
  });
});
