import { render, screen } from "@testing-library/svelte";
import { vi, describe, it, expect, beforeEach } from "vitest";

const { mockQueryState } = vi.hoisted(() => {
  const mockQueryState = { data: null as any, isPending: false };
  return { mockQueryState };
});

vi.mock("@tanstack/svelte-query", () => ({
  createQuery: () => ({
    get data() {
      return mockQueryState.data;
    },
    get isPending() {
      return mockQueryState.isPending;
    },
    get isError() {
      return false;
    },
  }),
}));

vi.mock("$lib/api/players", () => ({ getPlayers: vi.fn() }));
vi.mock("$lib/state/auth.svelte", () => ({
  auth: { isAdmin: false, isModerator: false },
}));
vi.mock("$lib/components/PlayerAvatar.svelte", () => ({ default: vi.fn() }));

import Page from "./+page.svelte";

function makePlayer(
  id: string,
  firstName: string,
  lastName: string,
  position: string,
) {
  return { id, firstName, lastName, position, shirtNumber: 1, active: true, user: null };
}

describe("Players list page — sorting and grouping", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockQueryState.isPending = false;
  });

  it("renders group headers for each position group present", () => {
    mockQueryState.data = [
      makePlayer("1", "Ali", "Zomer", "Spits"),
      makePlayer("2", "Ben", "Adams", "Keeper"),
      makePlayer("3", "Chris", "Berg", "Centrale verdediger"),
    ];

    render(Page);

    const headings = screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent?.trim());
    expect(headings).toContain("Aanvallers");
    expect(headings).toContain("Verdedigers");
    expect(headings).toContain("Keeper");
    expect(headings).not.toContain("Middenvelders");
  });

  it("shows strikers before midfielders before defenders before keeper", () => {
    mockQueryState.data = [
      makePlayer("1", "Ko", "de Keeper", "Keeper"),
      makePlayer("2", "Ve", "Verdediger", "Centrale verdediger"),
      makePlayer("3", "Mi", "Middenvelder", "Centrale middenvelder"),
      makePlayer("4", "Sp", "de Spits", "Spits"),
    ];

    render(Page);

    const names = screen
      .getAllByRole("link")
      .map((el) => el.textContent?.trim() ?? "");

    const spitsIdx = names.findIndex((n) => n.includes("de Spits"));
    const midIdx = names.findIndex((n) => n.includes("Middenvelder"));
    const verdIdx = names.findIndex((n) => n.includes("Verdediger"));
    const keeperIdx = names.findIndex((n) => n.includes("de Keeper"));

    expect(spitsIdx).toBeLessThan(midIdx);
    expect(midIdx).toBeLessThan(verdIdx);
    expect(verdIdx).toBeLessThan(keeperIdx);
  });

  it("sorts alphabetically by last name within the same position group", () => {
    mockQueryState.data = [
      makePlayer("1", "Jan", "Zwart", "Centrale middenvelder"),
      makePlayer("2", "Piet", "Aarts", "Centrale middenvelder"),
      makePlayer("3", "Kees", "Mulder", "Centrale middenvelder"),
    ];

    render(Page);

    const links = screen.getAllByRole("link");
    const names = links.map((el) => el.textContent?.trim() ?? "");

    const aartsIdx = names.findIndex((n) => n.includes("Aarts"));
    const mulderIdx = names.findIndex((n) => n.includes("Mulder"));
    const zwartIdx = names.findIndex((n) => n.includes("Zwart"));

    expect(aartsIdx).toBeLessThan(mulderIdx);
    expect(mulderIdx).toBeLessThan(zwartIdx);
  });

  it("shows a spinner while loading", () => {
    mockQueryState.isPending = true;
    mockQueryState.data = null;

    render(Page);

    expect(screen.queryAllByRole("link")).toHaveLength(0);
  });

  it("renders nothing when the player list is empty", () => {
    mockQueryState.data = [];

    render(Page);

    expect(screen.queryAllByRole("link")).toHaveLength(0);
    expect(screen.queryByText("Aanvallers")).not.toBeInTheDocument();
  });
});
