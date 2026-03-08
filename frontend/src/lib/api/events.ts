import type { MatchEvent, CreateMatchEventRequest, PlayerStats } from './types.js';
import { getPlayers } from './players.js';
import { getMatches } from './matches.js';

const STORAGE_KEY = 'noordpool_events';

// Match 1 (home 3-1): 3 goals + 3 assists + 1 yellow
// Match 2 (away 2-2): 2 goals + 1 assist + 2 yellows + 1 red
// Match 3 (home 4-0): 4 goals + 2 assists + 0 cards
const SEED_EVENTS: MatchEvent[] = [
	// Match 1: Noordpool 3-1 VV Helpman
	{ id: 'e1', matchId: '1', playerId: '7', eventType: 'goal', minute: 12 },
	{ id: 'e2', matchId: '1', playerId: '6', eventType: 'assist', minute: 12 },
	{ id: 'e3', matchId: '1', playerId: '6', eventType: 'goal', minute: 34 },
	{ id: 'e4', matchId: '1', playerId: '5', eventType: 'assist', minute: 34 },
	{ id: 'e5', matchId: '1', playerId: '3', eventType: 'yellow_card', minute: 55 },
	{ id: 'e6', matchId: '1', playerId: '5', eventType: 'goal', minute: 78 },
	{ id: 'e7', matchId: '1', playerId: '4', eventType: 'assist', minute: 78 },

	// Match 2: SC Stadspark 2-2 Noordpool
	{ id: 'e8', matchId: '2', playerId: '6', eventType: 'goal', minute: 23 },
	{ id: 'e9', matchId: '2', playerId: '7', eventType: 'assist', minute: 23 },
	{ id: 'e10', matchId: '2', playerId: '4', eventType: 'yellow_card', minute: 40 },
	{ id: 'e11', matchId: '2', playerId: '2', eventType: 'yellow_card', minute: 58 },
	{ id: 'e12', matchId: '2', playerId: '7', eventType: 'goal', minute: 67 },
	{ id: 'e13', matchId: '2', playerId: '3', eventType: 'red_card', minute: 80 },

	// Match 3: Noordpool 4-0 FC Lewenborg
	{ id: 'e14', matchId: '3', playerId: '7', eventType: 'goal', minute: 8 },
	{ id: 'e15', matchId: '3', playerId: '6', eventType: 'assist', minute: 8 },
	{ id: 'e16', matchId: '3', playerId: '6', eventType: 'goal', minute: 29 },
	{ id: 'e17', matchId: '3', playerId: '7', eventType: 'goal', minute: 51 },
	{ id: 'e18', matchId: '3', playerId: '5', eventType: 'assist', minute: 51 },
	{ id: 'e19', matchId: '3', playerId: '5', eventType: 'goal', minute: 72 }
];

function _load(): MatchEvent[] {
	const raw = localStorage.getItem(STORAGE_KEY);
	if (raw) return JSON.parse(raw);
	_save(SEED_EVENTS);
	return [...SEED_EVENTS];
}

function _save(events: MatchEvent[]): void {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(events));
}

export function getMatchEvents(matchId: string): MatchEvent[] {
	return _load()
		.filter((e) => e.matchId === matchId)
		.sort((a, b) => {
			if (a.minute !== b.minute) return a.minute - b.minute;
			// goal before assist at same minute
			const order: Record<string, number> = { goal: 0, assist: 1, yellow_card: 2, red_card: 3 };
			return (order[a.eventType] ?? 9) - (order[b.eventType] ?? 9);
		});
}

export function createMatchEvent(matchId: string, data: CreateMatchEventRequest): MatchEvent {
	const events = _load();
	const event: MatchEvent = {
		id: crypto.randomUUID(),
		matchId,
		playerId: data.playerId,
		eventType: data.eventType,
		minute: data.minute
	};
	events.push(event);
	_save(events);
	return event;
}

export function deleteMatchEvent(eventId: string): void {
	const events = _load().filter((e) => e.id !== eventId);
	_save(events);
}

export function getPlayerStats(playerId: string): PlayerStats {
	const events = _load();
	const completedMatches = getMatches().filter((m) => m.status === 'completed');
	const player = getPlayers().find((p) => p.id === playerId);

	// Appearances = number of completed matches (for active players, assume they played all)
	const appearances = player?.active ? completedMatches.length : 0;

	const playerEvents = events.filter((e) => e.playerId === playerId);

	return {
		playerId,
		appearances,
		goals: playerEvents.filter((e) => e.eventType === 'goal').length,
		assists: playerEvents.filter((e) => e.eventType === 'assist').length,
		yellowCards: playerEvents.filter((e) => e.eventType === 'yellow_card').length,
		redCards: playerEvents.filter((e) => e.eventType === 'red_card').length
	};
}

export function getLeaderboard(): {
	topScorers: (PlayerStats & { name: string; shirtNumber: number })[];
	topAssisters: (PlayerStats & { name: string; shirtNumber: number })[];
	mostCarded: (PlayerStats & { name: string; shirtNumber: number; totalCards: number })[];
} {
	const players = getPlayers().filter((p) => p.active);
	const allStats = players.map((p) => ({
		...getPlayerStats(p.id),
		name: p.name,
		shirtNumber: p.shirtNumber
	}));

	const topScorers = allStats
		.filter((s) => s.goals > 0)
		.sort((a, b) => b.goals - a.goals);

	const topAssisters = allStats
		.filter((s) => s.assists > 0)
		.sort((a, b) => b.assists - a.assists);

	const mostCarded = allStats
		.map((s) => ({ ...s, totalCards: s.yellowCards + s.redCards }))
		.filter((s) => s.totalCards > 0)
		.sort((a, b) => b.totalCards - a.totalCards);

	return { topScorers, topAssisters, mostCarded };
}
