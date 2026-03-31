import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Page from './+page.svelte';

function fakeJwt(payload: Record<string, unknown>): string {
	const header = btoa(JSON.stringify({ alg: 'HS256' }));
	const body = btoa(JSON.stringify(payload));
	return `${header}.${body}.fake-signature`;
}

vi.mock('$lib/api/auth', () => ({
	logout: vi.fn(),
	unlinkPlayer: vi.fn(),
}));

vi.mock('$lib/api/client', () => ({
	getToken: vi.fn(),
}));

vi.mock('$lib/state/auth.svelte', () => ({
	auth: {
		isAuthenticated: true,
		user: {
			id: 'user-1',
			email: 'test@example.com',
			firstName: 'Jan',
			lastName: 'de Vries',
			avatarUrl: null,
			roles: ['player'],
		},
		isAdmin: false,
		isModerator: false,
		setUser: vi.fn(),
		clear: vi.fn(),
	},
}));

vi.mock('$lib/state/pwa.svelte', () => ({
	pwa: { installable: false, install: vi.fn() },
}));

vi.mock('$lib/state/theme.svelte', () => ({
	theme: { isDark: false, toggle: vi.fn() },
}));

import { unlinkPlayer } from '$lib/api/auth';
import { getToken } from '$lib/api/client';
import { auth } from '$lib/state/auth.svelte';
import { goto } from '$app/navigation';

beforeEach(() => {
	vi.clearAllMocks();
});

describe('profile page — linked player', () => {
	beforeEach(() => {
		vi.mocked(getToken).mockReturnValue(
			fakeJwt({ sub: 'user-1', player_id: 'player-1' })
		);
	});

	it('shows unlink button when player is linked', () => {
		render(Page);
		expect(screen.getByText('Ontkoppelen')).toBeInTheDocument();
		expect(screen.getByText('Gekoppelde speler')).toBeInTheDocument();
	});

	it('does not show link prompt when player is linked', () => {
		render(Page);
		expect(screen.queryByText('Geen speler gekoppeld')).not.toBeInTheDocument();
	});

	it('calls unlinkPlayer and updates state on click', async () => {
		vi.mocked(unlinkPlayer).mockResolvedValue({
			user: { id: 'user-1', email: 'test@example.com', firstName: 'Jan', lastName: 'de Vries', avatarUrl: null, roles: ['player'] },
			token: fakeJwt({ sub: 'user-1' }),
		});

		render(Page);
		await fireEvent.click(screen.getByText('Ontkoppelen'));

		await waitFor(() => {
			expect(unlinkPlayer).toHaveBeenCalledOnce();
			expect(auth.setUser).toHaveBeenCalledWith(
				expect.objectContaining({ id: 'user-1' })
			);
		});
	});
});

describe('profile page — no linked player', () => {
	beforeEach(() => {
		vi.mocked(getToken).mockReturnValue(
			fakeJwt({ sub: 'user-1' })
		);
	});

	it('shows link prompt when no player is linked', () => {
		render(Page);
		expect(screen.getByText('Geen speler gekoppeld')).toBeInTheDocument();
		expect(screen.getByText('Koppelen')).toBeInTheDocument();
	});

	it('does not show unlink button when no player is linked', () => {
		render(Page);
		expect(screen.queryByText('Ontkoppelen')).not.toBeInTheDocument();
	});

	it('link points to link-player page with user name', () => {
		render(Page);
		const link = screen.getByText('Koppelen').closest('a');
		expect(link).toHaveAttribute('href', '/auth/link-player?name=Jan%20de%20Vries');
	});
});

describe('profile page — general', () => {
	beforeEach(() => {
		vi.mocked(getToken).mockReturnValue(
			fakeJwt({ sub: 'user-1' })
		);
	});

	it('shows user name and email', () => {
		render(Page);
		expect(screen.getByText('Jan de Vries')).toBeInTheDocument();
		expect(screen.getByText('test@example.com')).toBeInTheDocument();
	});

	it('logs out and navigates to home', async () => {
		render(Page);
		await fireEvent.click(screen.getByText('Uitloggen'));
		expect(auth.clear).toHaveBeenCalled();
		expect(goto).toHaveBeenCalledWith('/');
	});
});
