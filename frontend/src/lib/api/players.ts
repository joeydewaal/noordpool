import type { Player, CreatePlayerRequest, UpdatePlayerRequest } from './types.js';

const STORAGE_KEY = 'noordpool_players';

const SEED_PLAYERS: Player[] = [
	{ id: '1', userId: null, name: 'Jan de Vries', shirtNumber: 1, position: 'goalkeeper', active: true },
	{ id: '2', userId: null, name: 'Pieter van Dijk', shirtNumber: 4, position: 'defender', active: true },
	{ id: '3', userId: null, name: 'Kees Bakker', shirtNumber: 5, position: 'defender', active: true },
	{ id: '4', userId: null, name: 'Willem Jansen', shirtNumber: 8, position: 'midfielder', active: true },
	{ id: '5', userId: null, name: 'Bram de Boer', shirtNumber: 10, position: 'midfielder', active: true },
	{ id: '6', userId: null, name: 'Lars Visser', shirtNumber: 7, position: 'forward', active: true },
	{ id: '7', userId: null, name: 'Daan Mulder', shirtNumber: 9, position: 'forward', active: true },
	{ id: '8', userId: null, name: 'Tom van den Berg', shirtNumber: 3, position: 'defender', active: false }
];

function _load(): Player[] {
	const raw = localStorage.getItem(STORAGE_KEY);
	if (raw) return JSON.parse(raw);
	_save(SEED_PLAYERS);
	return [...SEED_PLAYERS];
}

function _save(players: Player[]): void {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(players));
}

export function getPlayers(): Player[] {
	return _load();
}

export function getPlayer(id: string): Player | null {
	return _load().find((p) => p.id === id) ?? null;
}

export function createPlayer(data: CreatePlayerRequest): Player {
	const players = _load();
	const player: Player = {
		id: crypto.randomUUID(),
		userId: null,
		name: data.name,
		shirtNumber: data.shirtNumber,
		position: data.position,
		active: true
	};
	players.push(player);
	_save(players);
	return player;
}

export function updatePlayer(id: string, data: UpdatePlayerRequest): Player {
	const players = _load();
	const index = players.findIndex((p) => p.id === id);
	if (index === -1) throw new Error('Player not found');
	players[index] = { ...players[index], ...data };
	_save(players);
	return players[index];
}
