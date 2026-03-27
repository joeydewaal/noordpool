import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Page from './+page.svelte';

// Mock the auth API
vi.mock('$lib/api/auth', () => ({
	findPlayer: vi.fn(),
	register: vi.fn(),
}));

// Mock the auth state
vi.mock('$lib/state/auth.svelte', () => ({
	auth: { setUser: vi.fn() },
}));

import { findPlayer, register } from '$lib/api/auth';
import { goto } from '$app/navigation';

const mockPlayer = {
	id: 'player-uuid-1',
	name: 'Jan de Vries',
	shirtNumber: 7,
	position: 'midfielder' as const,
};

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(findPlayer).mockResolvedValue([]);
	vi.mocked(register).mockResolvedValue({
		user: { id: 'user-1', email: 'test@example.com', name: 'Test', avatarUrl: null, roles: ['player'] },
		token: 'mock-token',
	});
});

describe('find-player flow', () => {
	it('does not call findPlayer when name is too short', async () => {
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'J' } });
		await fireEvent.blur(nameInput);
		expect(findPlayer).not.toHaveBeenCalled();
	});

	it('calls findPlayer on name blur when name is long enough', async () => {
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => expect(findPlayer).toHaveBeenCalledWith('Jan'));
	});

	it('shows match card when findPlayer returns results', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => expect(screen.getByText(/jan de vries/i)).toBeInTheDocument());
		expect(screen.getByText(/ja, dat ben ik/i)).toBeInTheDocument();
	});

	it('shows no match card when findPlayer returns empty', async () => {
		vi.mocked(findPlayer).mockResolvedValue([]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => expect(findPlayer).toHaveBeenCalled());
		expect(screen.queryByText(/ja, dat ben ik/i)).not.toBeInTheDocument();
	});

	it('shows linked badge after confirming a match', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => screen.getByText(/ja, dat ben ik/i));
		await fireEvent.click(screen.getByText(/ja, dat ben ik/i));
		// Badge text is split across elements, so check parts individually
		expect(screen.getByText(/gekoppeld aan/i)).toBeInTheDocument();
		expect(screen.getByText('Jan de Vries', { selector: 'strong' })).toBeInTheDocument();
		expect(screen.queryByText(/ja, dat ben ik/i)).not.toBeInTheDocument();
	});

	it('dismisses match card on "Nee" click', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => screen.getByText(/nee, ik ben dit niet/i));
		await fireEvent.click(screen.getByText(/nee, ik ben dit niet/i));
		expect(screen.queryByText(/ja, dat ben ik/i)).not.toBeInTheDocument();
	});

	it('does not call findPlayer again after dismissing', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => screen.getByText(/nee, ik ben dit niet/i));
		await fireEvent.click(screen.getByText(/nee, ik ben dit niet/i));

		// Blur again — should not trigger another lookup
		await fireEvent.blur(nameInput);
		expect(findPlayer).toHaveBeenCalledTimes(1);
	});

	it('removes linked badge on "Ontkoppelen" click', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		await waitFor(() => screen.getByText(/ja, dat ben ik/i));
		await fireEvent.click(screen.getByText(/ja, dat ben ik/i));
		await waitFor(() => screen.getByText(/ontkoppelen/i));
		await fireEvent.click(screen.getByText(/ontkoppelen/i));
		expect(screen.queryByText(/gekoppeld aan/i)).not.toBeInTheDocument();
		expect(screen.queryByText('Jan de Vries', { selector: 'strong' })).not.toBeInTheDocument();
	});
});

describe('register form submission', () => {
	it('submits with playerId when a player is linked', async () => {
		vi.mocked(findPlayer).mockResolvedValue([mockPlayer]);
		render(Page);

		// Fill form
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Jan de Vries' } });
		await fireEvent.blur(screen.getByLabelText(/naam/i));
		await waitFor(() => screen.getByText(/ja, dat ben ik/i));
		await fireEvent.click(screen.getByText(/ja, dat ben ik/i));

		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'jan@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'geheim123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'geheim123' } });

		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));

		await waitFor(() => expect(register).toHaveBeenCalledWith(expect.objectContaining({
			playerId: mockPlayer.id,
			email: 'jan@example.com',
		})));
	});

	it('submits without playerId when no player is linked', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Nieuw Iemand' } });
		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'nieuw@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'test123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'test123' } });

		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));

		await waitFor(() => expect(register).toHaveBeenCalledWith(expect.not.objectContaining({
			playerId: expect.anything(),
		})));
	});

	it('shows error when passwords do not match', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'abc123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'xyz999' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));
		expect(screen.getByText(/wachtwoorden komen niet overeen/i)).toBeInTheDocument();
		expect(register).not.toHaveBeenCalled();
	});

	it('navigates to home on successful registration', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Test' } });
		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'test@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'test123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'test123' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));
		await waitFor(() => expect(goto).toHaveBeenCalledWith('/'));
	});

	it('shows error on registration failure', async () => {
		vi.mocked(register).mockRejectedValue(new Error('conflict'));
		render(Page);
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Test' } });
		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'test@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'test123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'test123' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));
		await waitFor(() => expect(screen.getByText(/registratie mislukt/i)).toBeInTheDocument());
	});
});
