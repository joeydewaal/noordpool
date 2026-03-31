import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Page from './+page.svelte';

const mockUser = {
	id: 'user-1',
	email: 'test@example.com',
	firstName: 'Jan',
	lastName: 'de Vries',
	avatarUrl: null,
	roles: ['player'] as string[],
};

vi.mock('$lib/api/auth', () => ({
	logout: vi.fn(),
	unlinkPlayer: vi.fn(),
}));

vi.mock('$lib/state/auth.svelte', () => ({
	auth: {
		get isAuthenticated() { return true; },
		get user() { return mockUser; },
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
import { auth } from '$lib/state/auth.svelte';
import { goto } from '$app/navigation';

beforeEach(() => {
	vi.clearAllMocks();
	mockUser.roles = ['player'];
});

describe('profile page — linked player', () => {
	it('shows unlink button when user has player role', () => {
		render(Page);
		expect(screen.getByText('Ontkoppelen')).toBeInTheDocument();
		expect(screen.getByText('Gekoppelde speler')).toBeInTheDocument();
	});

	it('does not show link prompt when user has player role', () => {
		render(Page);
		expect(screen.queryByText('Geen speler gekoppeld')).not.toBeInTheDocument();
	});

	it('calls unlinkPlayer and updates state on click', async () => {
		vi.mocked(unlinkPlayer).mockResolvedValue({
			user: { id: 'user-1', email: 'test@example.com', firstName: 'Jan', lastName: 'de Vries', avatarUrl: null, roles: [] },
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
