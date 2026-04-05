import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';

const { mockUser, mockAuth, mockPlayerData, mockUpdatePlayer } = vi.hoisted(() => {
	const mockUser = {
		id: 'user-1',
		email: 'test@example.com',
		firstName: 'Jan',
		lastName: 'de Vries',
		avatarUrl: null as string | null,
		playerId: 'player-1' as string | null,
		roles: ['player'] as string[],
	};
	const mockAuth = {
		get isAuthenticated() { return true; },
		get user() { return mockUser; },
		get playerId() { return mockUser.playerId; },
		isAdmin: false,
		isModerator: false,
		setUser: vi.fn(),
		clear: vi.fn(),
	};
	const mockPlayerData = {
		data: null as any,
	};
	const mockUpdatePlayer = vi.fn();
	return { mockUser, mockAuth, mockPlayerData, mockUpdatePlayer };
});

vi.mock('$lib/api/auth', () => ({
	logout: vi.fn(),
	unlinkPlayer: vi.fn(),
}));

vi.mock('$lib/state/auth.svelte', () => ({
	auth: mockAuth,
}));

vi.mock('$lib/state/pwa.svelte', () => ({
	pwa: { installable: false, install: vi.fn() },
}));

vi.mock('$lib/state/theme.svelte', () => ({
	theme: { isDark: false, toggle: vi.fn() },
}));

vi.mock('$lib/api/players', () => ({
	getPlayer: vi.fn(),
	updatePlayer: mockUpdatePlayer,
}));

vi.mock('@tanstack/svelte-query', () => ({
	createQuery: () => ({
		get data() { return mockPlayerData.data; },
		get isPending() { return false; },
		get isError() { return false; },
	}),
	createMutation: (optsFn: any) => {
		const opts = optsFn();
		return {
			get mutate() {
				return (data: any) => {
					opts.mutationFn(data).then(() => opts.onSuccess?.());
				};
			},
			get isPending() { return false; },
		};
	},
	useQueryClient: () => ({
		invalidateQueries: vi.fn(),
	}),
}));

import Page from './+page.svelte';
import { unlinkPlayer } from '$lib/api/auth';
import { auth } from '$lib/state/auth.svelte';
import { goto } from '$app/navigation';

const testPlayer = {
	id: 'player-1',
	userId: 'user-1',
	firstName: 'Jan',
	lastName: 'de Vries',
	shirtNumber: 10,
	position: 'Centrale middenvelder' as const,
	active: true,
};

beforeEach(() => {
	vi.clearAllMocks();
	mockUser.roles = ['player'];
	mockUser.playerId = 'player-1';
	mockPlayerData.data = testPlayer;
	mockUpdatePlayer.mockResolvedValue(testPlayer);
});

describe('profile page — linked player', () => {
	it('shows unlink button when user has player role', () => {
		render(Page);
		expect(screen.getByText('Ontkoppelen')).toBeInTheDocument();
		expect(screen.getByText('Gekoppelde speler')).toBeInTheDocument();
	});

	it('shows player info with shirt number and position', () => {
		render(Page);
		expect(screen.getByText('10')).toBeInTheDocument();
		expect(screen.getByText('Centrale middenvelder')).toBeInTheDocument();
	});

	it('shows edit button for linked player', () => {
		render(Page);
		expect(screen.getByText('Bewerken')).toBeInTheDocument();
	});

	it('shows shirt number and position fields when editing', async () => {
		render(Page);
		await fireEvent.click(screen.getByText('Bewerken'));

		expect(screen.getByLabelText('Rugnummer')).toBeInTheDocument();
		expect(screen.getByLabelText('Positie')).toBeInTheDocument();
		// Should not show name or active fields
		expect(screen.queryByLabelText('Voornaam')).not.toBeInTheDocument();
		expect(screen.queryByLabelText('Actief')).not.toBeInTheDocument();
	});

	it('submits only shirt number and position', async () => {
		render(Page);
		await fireEvent.click(screen.getByText('Bewerken'));
		await fireEvent.click(screen.getByText('Opslaan'));

		expect(mockUpdatePlayer).toHaveBeenCalledWith('player-1', {
			shirtNumber: 10,
			position: 'Centrale middenvelder',
		});
	});

	it('does not show link prompt when user has player role', () => {
		render(Page);
		expect(screen.queryByText('Geen speler gekoppeld')).not.toBeInTheDocument();
	});

	it('calls unlinkPlayer and updates state on click', async () => {
		vi.mocked(unlinkPlayer).mockResolvedValue({
			user: { id: 'user-1', email: 'test@example.com', firstName: 'Jan', lastName: 'de Vries', avatarUrl: null, playerId: null, roles: [] },
			token: 'mock-token',
		});

		render(Page);
		await fireEvent.click(screen.getByText('Ontkoppelen'));

		await waitFor(() => {
			expect(unlinkPlayer).toHaveBeenCalledOnce();
			expect(auth.setUser).toHaveBeenCalledWith(
				expect.objectContaining({ id: 'user-1', roles: [] })
			);
		});
	});
});

describe('profile page — no linked player', () => {
	beforeEach(() => {
		mockUser.roles = [];
		mockUser.playerId = null;
		mockPlayerData.data = null;
	});

	it('shows link prompt when user has no player role', () => {
		render(Page);
		expect(screen.getByText('Geen speler gekoppeld')).toBeInTheDocument();
		expect(screen.getByText('Koppelen')).toBeInTheDocument();
	});

	it('does not show unlink button when user has no player role', () => {
		render(Page);
		expect(screen.queryByText('Ontkoppelen')).not.toBeInTheDocument();
	});

	it('link points to link-player page', () => {
		render(Page);
		const link = screen.getByText('Koppelen').closest('a');
		expect(link).toHaveAttribute('href', '/auth/link-player');
	});
});

describe('profile page — general', () => {
	it('shows user name and email', () => {
		render(Page);
		expect(screen.getByRole('heading', { name: 'Jan de Vries' })).toBeInTheDocument();
		expect(screen.getByText('test@example.com')).toBeInTheDocument();
	});

	it('logs out and navigates to home', async () => {
		render(Page);
		await fireEvent.click(screen.getByText('Uitloggen'));
		expect(auth.clear).toHaveBeenCalled();
		expect(goto).toHaveBeenCalledWith('/');
	});
});
