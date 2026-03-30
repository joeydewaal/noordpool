import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import Page from './+page.svelte';

vi.mock('$lib/api/auth', () => ({
	register: vi.fn(),
}));

vi.mock('$lib/state/auth.svelte', () => ({
	auth: { setUser: vi.fn() },
}));

import { register } from '$lib/api/auth';
import { goto } from '$app/navigation';

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(register).mockResolvedValue({
		user: { id: 'user-1', email: 'test@example.com', name: 'Test', avatarUrl: null, roles: ['player'] },
		token: 'mock-token',
	});
});

describe('register form submission', () => {
	it('submits without playerId', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Jan de Vries' } });
		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'jan@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'geheim123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'geheim123' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));

		await waitFor(() => expect(register).toHaveBeenCalledWith({
			name: 'Jan de Vries',
			email: 'jan@example.com',
			password: 'geheim123',
		}));
	});

	it('navigates to link-player page with name after successful registration', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/naam/i), { target: { value: 'Jan de Vries' } });
		await fireEvent.input(screen.getByLabelText(/^e-mail/i), { target: { value: 'jan@example.com' } });
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'geheim123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'geheim123' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));

		await waitFor(() => expect(goto).toHaveBeenCalledWith('/auth/link-player?name=Jan%20de%20Vries'));
	});

	it('shows error when passwords do not match', async () => {
		render(Page);
		await fireEvent.input(screen.getByLabelText(/^wachtwoord$/i), { target: { value: 'abc123' } });
		await fireEvent.input(screen.getByLabelText(/bevestig wachtwoord/i), { target: { value: 'xyz999' } });
		await fireEvent.submit(screen.getByRole('button', { name: /registreren/i }));
		expect(screen.getByText(/wachtwoorden komen niet overeen/i)).toBeInTheDocument();
		expect(register).not.toHaveBeenCalled();
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

	it('does not call findPlayer on name blur', async () => {
		const findPlayer = vi.fn();
		render(Page);
		const nameInput = screen.getByLabelText(/naam/i);
		await fireEvent.input(nameInput, { target: { value: 'Jan' } });
		await fireEvent.blur(nameInput);
		expect(findPlayer).not.toHaveBeenCalled();
	});
});
