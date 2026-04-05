import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';

const { mockAuth, mockQueryState } = vi.hoisted(() => {
    const mockAuth = {
        isAdmin: false,
        isModerator: false,
        playerId: null as string | null,
    };
    const mockQueryState = {
        data: null as any,
        pending: false,
    };
    return { mockAuth, mockQueryState };
});

vi.mock('$app/state', () => ({
    page: { params: { id: 'test-player-id' } },
}));

vi.mock('$app/navigation', () => ({
    goto: vi.fn(),
}));

vi.mock('$lib/state/auth.svelte', () => ({
    auth: mockAuth,
}));

vi.mock('$lib/api/players', () => ({
    getPlayer: vi.fn(),
    updatePlayer: vi.fn(),
}));

vi.mock('@tanstack/svelte-query', () => ({
    createQuery: () => ({
        get data() { return mockQueryState.data; },
        get isPending() { return mockQueryState.pending; },
        get isError() { return false; },
    }),
    createMutation: () => ({
        mutate: vi.fn(),
        get isPending() { return false; },
    }),
    useQueryClient: () => ({
        invalidateQueries: vi.fn(),
    }),
}));

import Page from './+page.svelte';

const testPlayer = {
    id: 'test-player-id',
    userId: null,
    firstName: 'Jan',
    lastName: 'de Boer',
    shirtNumber: 10,
    position: 'Centrale middenvelder' as const,
    active: true,
};

describe('Player edit page', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mockAuth.isAdmin = false;
        mockAuth.isModerator = false;
        mockAuth.playerId = null;
        mockQueryState.data = testPlayer;
        mockQueryState.pending = false;
    });

    it('shows all fields for admin', () => {
        mockAuth.isAdmin = true;

        render(Page);

        expect(screen.getByLabelText('Voornaam')).toBeInTheDocument();
        expect(screen.getByLabelText('Achternaam')).toBeInTheDocument();
        expect(screen.getByLabelText('Rugnummer')).toBeInTheDocument();
        expect(screen.getByLabelText('Positie')).toBeInTheDocument();
        expect(screen.getByLabelText('Actief')).toBeInTheDocument();
    });

    it('shows access denied for regular player', () => {
        mockAuth.playerId = 'test-player-id';

        render(Page);

        expect(screen.getByText(/Geen toegang/)).toBeInTheDocument();
        expect(screen.queryByLabelText('Rugnummer')).not.toBeInTheDocument();
    });

    it('shows all fields for moderator', () => {
        mockAuth.isModerator = true;

        render(Page);

        expect(screen.getByLabelText('Voornaam')).toBeInTheDocument();
        expect(screen.getByLabelText('Rugnummer')).toBeInTheDocument();
    });
});
