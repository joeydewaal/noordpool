import type { Match, CreateMatchRequest, UpdateMatchRequest } from './types.js';

const STORAGE_KEY = 'noordpool_matches';

const SEED_MATCHES: Match[] = [
	{
		id: '1',
		opponent: 'VV Helpman',
		location: 'Sportpark Noordpool',
		dateTime: '2026-02-15T14:00:00',
		homeAway: 'home',
		status: 'completed',
		homeScore: 3,
		awayScore: 1,
		createdAt: '2026-02-01T10:00:00'
	},
	{
		id: '2',
		opponent: 'SC Stadspark',
		location: 'Stadspark Veld 2',
		dateTime: '2026-02-22T14:30:00',
		homeAway: 'away',
		status: 'completed',
		homeScore: 2,
		awayScore: 2,
		createdAt: '2026-02-08T10:00:00'
	},
	{
		id: '3',
		opponent: 'FC Lewenborg',
		location: 'Sportpark Noordpool',
		dateTime: '2026-03-01T14:00:00',
		homeAway: 'home',
		status: 'completed',
		homeScore: 4,
		awayScore: 0,
		createdAt: '2026-02-15T10:00:00'
	},
	{
		id: '4',
		opponent: 'Be Quick 1887',
		location: 'Sportpark Kardinge',
		dateTime: '2026-03-15T14:30:00',
		homeAway: 'away',
		status: 'scheduled',
		homeScore: null,
		awayScore: null,
		createdAt: '2026-03-01T10:00:00'
	},
	{
		id: '5',
		opponent: 'VV Groningen',
		location: 'Sportpark Noordpool',
		dateTime: '2026-03-22T14:00:00',
		homeAway: 'home',
		status: 'scheduled',
		homeScore: null,
		awayScore: null,
		createdAt: '2026-03-01T10:00:00'
	},
	{
		id: '6',
		opponent: 'Velocitas 1897',
		location: 'Sportpark Vinkhuizen',
		dateTime: '2026-03-29T14:30:00',
		homeAway: 'away',
		status: 'scheduled',
		homeScore: null,
		awayScore: null,
		createdAt: '2026-03-01T10:00:00'
	}
];

function _load(): Match[] {
	const raw = localStorage.getItem(STORAGE_KEY);
	if (raw) return JSON.parse(raw);
	_save(SEED_MATCHES);
	return [...SEED_MATCHES];
}

function _save(matches: Match[]): void {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(matches));
}

export function getMatches(): Match[] {
	return _load().sort((a, b) => b.dateTime.localeCompare(a.dateTime));
}

export function getMatch(id: string): Match | null {
	return _load().find((m) => m.id === id) ?? null;
}

export function getUpcomingMatches(limit?: number): Match[] {
	const upcoming = _load()
		.filter((m) => m.status === 'scheduled')
		.sort((a, b) => a.dateTime.localeCompare(b.dateTime));
	return limit ? upcoming.slice(0, limit) : upcoming;
}

export function getRecentResults(limit?: number): Match[] {
	const results = _load()
		.filter((m) => m.status === 'completed')
		.sort((a, b) => b.dateTime.localeCompare(a.dateTime));
	return limit ? results.slice(0, limit) : results;
}

export function createMatch(data: CreateMatchRequest): Match {
	const matches = _load();
	const match: Match = {
		id: crypto.randomUUID(),
		opponent: data.opponent,
		location: data.location,
		dateTime: data.dateTime,
		homeAway: data.homeAway,
		status: 'scheduled',
		homeScore: null,
		awayScore: null,
		createdAt: new Date().toISOString()
	};
	matches.push(match);
	_save(matches);
	return match;
}

export function updateMatch(id: string, data: UpdateMatchRequest): Match {
	const matches = _load();
	const index = matches.findIndex((m) => m.id === id);
	if (index === -1) throw new Error('Match not found');
	matches[index] = { ...matches[index], ...data };
	_save(matches);
	return matches[index];
}
