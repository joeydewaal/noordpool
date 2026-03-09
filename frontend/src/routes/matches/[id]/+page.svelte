<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getMatch } from '$lib/api/matches.js';
	import { getMatchEvents, createMatchEvent, deleteMatchEvent } from '$lib/api/events.js';
	import { getPlayers } from '$lib/api/players.js';
	import type { Match, MatchEvent, Player, EventType } from '$lib/api/types.js';

	let match: Match | null = $state(null);
	let events: MatchEvent[] = $state([]);
	let players: Player[] = $state([]);

	let newPlayerId = $state('');
	let newEventType: EventType = $state('goal');
	let newMinute = $state(1);

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	const eventLabels: Record<EventType, string> = {
		goal: 'Doelpunt',
		assist: 'Assist',
		yellow_card: 'Gele kaart',
		red_card: 'Rode kaart'
	};

	const eventIcons: Record<EventType, string> = {
		goal: '\u26BD',
		assist: '\uD83D\uDC5F',
		yellow_card: '\uD83D\uDFE8',
		red_card: '\uD83D\uDFE5'
	};

	function playerName(playerId: string): string {
		return players.find((p) => p.id === playerId)?.name ?? 'Onbekend';
	}

	async function reloadEvents() {
		if (match) events = await getMatchEvents(match.id);
	}

	async function handleAddEvent() {
		if (!match || !newPlayerId) return;
		await createMatchEvent(match.id, {
			playerId: newPlayerId,
			eventType: newEventType,
			minute: newMinute
		});
		await reloadEvents();
		newPlayerId = '';
		newEventType = 'goal';
		newMinute = 1;
	}

	async function handleDeleteEvent(eventId: string) {
		if (!match) return;
		await deleteMatchEvent(match.id, eventId);
		await reloadEvents();
	}

	function formatDate(dateTime: string): string {
		return new Date(dateTime).toLocaleDateString('nl-NL', {
			weekday: 'long',
			day: 'numeric',
			month: 'long',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	onMount(async () => {
		[match, players] = await Promise.all([
			getMatch(page.params.id),
			getPlayers()
		]);
		if (match) events = await getMatchEvents(match.id);
	});
</script>

{#if match}
	<div class="max-w-lg">
		<a href="/matches" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Alle wedstrijden</a>
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center justify-between mb-2">
				<h1 class="text-2xl font-bold text-gray-900">vs {match.opponent}</h1>
				<span class="text-xs font-medium px-2.5 py-1 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
					{match.homeAway === 'home' ? 'thuis' : 'uit'}
				</span>
			</div>

			<div class="text-sm text-gray-500 space-y-1 mb-4">
				<div>{formatDate(match.dateTime)}</div>
				<div>{match.location}</div>
				<div>
					Status:
					<span class="font-medium {match.status === 'completed' ? 'text-green-700' : match.status === 'cancelled' ? 'text-red-700' : 'text-blue-700'}">
						{match.status === 'scheduled' ? 'gepland' : match.status === 'completed' ? 'gespeeld' : 'afgelast'}
					</span>
				</div>
			</div>

			{#if match.status === 'completed' && match.homeScore !== null}
				<div class="bg-gray-50 rounded-lg p-4 text-center">
					{#if match.homeAway === 'home'}
						<div class="text-lg">
							<span class="font-bold">Noordpool {match.homeScore}</span>
							<span class="text-gray-400 mx-2">-</span>
							<span class="font-bold">{match.awayScore} {match.opponent}</span>
						</div>
					{:else}
						<div class="text-lg">
							<span class="font-bold">{match.opponent} {match.homeScore}</span>
							<span class="text-gray-400 mx-2">-</span>
							<span class="font-bold">{match.awayScore} Noordpool</span>
						</div>
					{/if}
				</div>
			{/if}

			{#if match.status === 'completed'}
				<div class="mt-6 pt-4 border-t border-gray-100">
					<h2 class="text-lg font-bold text-gray-900 mb-3">Wedstrijdverloop</h2>

					{#if events.length === 0}
						<p class="text-sm text-gray-400">Geen gebeurtenissen geregistreerd.</p>
					{:else}
						<div class="space-y-2">
							{#each events as event}
								<div class="flex items-center gap-3 text-sm">
									<span class="inline-flex items-center justify-center w-10 h-6 bg-gray-100 text-gray-700 font-mono text-xs rounded">
										{event.minute}'
									</span>
									<span class="text-base">{eventIcons[event.eventType]}</span>
									<span class="font-medium text-gray-900">{playerName(event.playerId)}</span>
									<span class="text-gray-500">{eventLabels[event.eventType]}</span>
									{#if canManage}
										<button
											onclick={() => handleDeleteEvent(event.id)}
											class="ml-auto text-red-400 hover:text-red-600 text-xs"
											title="Verwijderen"
										>&times;</button>
									{/if}
								</div>
							{/each}
						</div>
					{/if}

					{#if canManage}
						<div class="mt-4 pt-3 border-t border-gray-50">
							<h3 class="text-sm font-semibold text-gray-700 mb-2">Gebeurtenis toevoegen</h3>
							<form onsubmit={(e) => { e.preventDefault(); handleAddEvent(); }} class="flex flex-wrap gap-2 items-end">
								<select bind:value={newPlayerId} class="border border-gray-300 rounded px-2 py-1.5 text-sm flex-1 min-w-[140px]">
									<option value="">Speler...</option>
									{#each players.filter(p => p.active) as p}
										<option value={p.id}>{p.name}</option>
									{/each}
								</select>
								<select bind:value={newEventType} class="border border-gray-300 rounded px-2 py-1.5 text-sm">
									<option value="goal">Doelpunt</option>
									<option value="assist">Assist</option>
									<option value="yellow_card">Gele kaart</option>
									<option value="red_card">Rode kaart</option>
								</select>
								<input type="number" bind:value={newMinute} min="1" max="120" class="border border-gray-300 rounded px-2 py-1.5 text-sm w-16" placeholder="min" />
								<button
									type="submit"
									disabled={!newPlayerId}
									class="bg-primary hover:bg-primary-light disabled:opacity-50 text-white text-sm font-medium px-3 py-1.5 rounded transition-colors"
								>
									Toevoegen
								</button>
							</form>
						</div>
					{/if}
				</div>
			{/if}

			{#if canManage}
				<div class="mt-6 pt-4 border-t border-gray-100">
					<a
						href="/matches/{match.id}/edit"
						class="bg-primary hover:bg-primary-light text-white text-sm font-medium px-4 py-2 rounded-lg transition-colors"
					>
						Bewerken
					</a>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<p class="text-gray-500">Wedstrijd niet gevonden.</p>
{/if}
