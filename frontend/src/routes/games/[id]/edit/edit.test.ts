import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import type { Game } from "$lib/api/types";

const { mockAuth, mockState, updateMutate } = vi.hoisted(() => {
  const mockAuth = { isAdmin: false, isModerator: false };
  const mockState = { data: null as Game | null };
  const updateMutate = vi.fn();
  return { mockAuth, mockState, updateMutate };
});

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

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: () => ({
    get data() {
      return mockState.data;
    },
    get isPending() {
      return false;
    },
    get isError() {
      return false;
    },
  }),
  createMutation: () => ({
    mutate: updateMutate,
    get isPending() {
      return false;
    },
  }),
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

import Page from "./+page.svelte";

function makeGame(overrides: Partial<Game> = {}): Game {
  return {
    id: "game-1",
    opponent: "PSV",
    location: "Stadium",
    dateTime: "2026-06-15T18:00:00Z",
    homeAway: "home",
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
  mockState.data = null;
});

describe("Game edit page", () => {
  it("shows access denied for regular player", () => {
    mockState.data = makeGame();
    render(Page);
    expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
  });

  it("populates the datetime-local input with a local-time value (no Z suffix)", async () => {
    mockAuth.isModerator = true;
    mockState.data = makeGame({ dateTime: "2026-06-15T18:00:00Z" });

    render(Page);

    const input = (await screen.findByLabelText(
      "Datum & tijd",
    )) as HTMLInputElement;
    // Must match the strict datetime-local format `YYYY-MM-DDTHH:mm`,
    // otherwise the browser drops the value silently.
    expect(input.value).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/);
    expect(input.value).not.toContain("Z");

    // And it must round-trip back to the same instant.
    expect(new Date(input.value).toISOString()).toBe(
      "2026-06-15T18:00:00.000Z",
    );
  });

  it("submits dateTime as a UTC ISO string regardless of local TZ", async () => {
    mockAuth.isModerator = true;
    mockState.data = makeGame({
      opponent: "Ajax",
      location: "JC Arena",
      dateTime: "2026-06-15T18:00:00Z",
    });

    render(Page);

    const form = await screen.findByRole("button", { name: /opslaan/i });
    await fireEvent.click(form);

    await waitFor(() => expect(updateMutate).toHaveBeenCalled());
    const [payload] = updateMutate.mock.calls[0];
    expect(payload.opponent).toBe("Ajax");
    expect(payload.location).toBe("JC Arena");
    // Backend wants RFC 3339 UTC; Date.toISOString() always ends in Z.
    expect(payload.dateTime).toMatch(/Z$/);
    expect(new Date(payload.dateTime).toISOString()).toBe(
      "2026-06-15T18:00:00.000Z",
    );
  });

  it("preserves a user-edited datetime through the round-trip", async () => {
    mockAuth.isModerator = true;
    mockState.data = makeGame({ dateTime: "2026-06-15T18:00:00Z" });

    render(Page);

    const input = (await screen.findByLabelText(
      "Datum & tijd",
    )) as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "2026-07-01T20:30" } });

    await fireEvent.click(screen.getByRole("button", { name: /opslaan/i }));

    await waitFor(() => expect(updateMutate).toHaveBeenCalled());
    const [payload] = updateMutate.mock.calls[0];
    // "2026-07-01T20:30" interpreted as local time, then converted to UTC.
    expect(payload.dateTime).toBe(new Date("2026-07-01T20:30").toISOString());
  });
});
