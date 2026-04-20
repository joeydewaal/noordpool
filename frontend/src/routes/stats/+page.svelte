<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { getLeaderboard } from "$lib/api/events";
  import Spinner from "$lib/components/Spinner.svelte";

  const leaderboardQuery = createQuery(() => ({
    queryKey: ["leaderboard"],
    queryFn: getLeaderboard,
    staleTime: 3 * 60_000,
  }));
</script>

<div class="max-w-2xl space-y-6">
  <h1 class="text-2xl font-bold">Statistieken</h1>

  {#if leaderboardQuery.isPending}
    <Spinner />
  {:else if leaderboardQuery.isError}
    <p class="text-error-500 text-sm">Kon statistieken niet laden</p>
  {:else}
    <div class="card p-6">
      <h2 class="text-lg font-bold mb-3">Topscorers</h2>
      {#if !leaderboardQuery.data || leaderboardQuery.data.topScorers.length === 0}
        <p class="text-sm text-surface-400">Nog geen doelpunten.</p>
      {:else}
        <div class="table-wrap">
          <table class="table">
            <thead>
              <tr>
                <th class="w-8">#</th>
                <th>Speler</th>
                <th class="w-12 text-center">Nr</th>
                <th class="w-16 text-center">Goals</th>
              </tr>
            </thead>
            <tbody>
              {#each leaderboardQuery.data.topScorers as player, i}
                <tr>
                  <td class="text-surface-400">{i + 1}</td>
                  <td
                    ><a
                      href="/players/{player.playerId}"
                      class="text-primary-500 hover:underline font-medium"
                      >{player.firstName} {player.lastName}</a
                    ></td
                  >
                  <td class="text-center text-surface-400"
                    >{player.shirtNumber}</td
                  >
                  <td class="text-center font-bold">{player.goals}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <div class="card p-6">
      <h2 class="text-lg font-bold mb-3">Meeste assists</h2>
      {#if !leaderboardQuery.data || leaderboardQuery.data.topAssisters.length === 0}
        <p class="text-sm text-surface-400">Nog geen assists.</p>
      {:else}
        <div class="table-wrap">
          <table class="table">
            <thead>
              <tr>
                <th class="w-8">#</th>
                <th>Speler</th>
                <th class="w-12 text-center">Nr</th>
                <th class="w-16 text-center">Assists</th>
              </tr>
            </thead>
            <tbody>
              {#each leaderboardQuery.data.topAssisters as player, i}
                <tr>
                  <td class="text-surface-400">{i + 1}</td>
                  <td
                    ><a
                      href="/players/{player.playerId}"
                      class="text-primary-500 hover:underline font-medium"
                      >{player.firstName} {player.lastName}</a
                    ></td
                  >
                  <td class="text-center text-surface-400"
                    >{player.shirtNumber}</td
                  >
                  <td class="text-center font-bold">{player.assists}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>

    <div class="card p-6">
      <h2 class="text-lg font-bold mb-3">Meeste kaarten</h2>
      {#if !leaderboardQuery.data || leaderboardQuery.data.mostCarded.length === 0}
        <p class="text-sm text-surface-400">Nog geen kaarten.</p>
      {:else}
        <div class="table-wrap">
          <table class="table">
            <thead>
              <tr>
                <th class="w-8">#</th>
                <th>Speler</th>
                <th class="w-12 text-center">Nr</th>
                <th class="w-12 text-center">Geel</th>
                <th class="w-12 text-center">Rood</th>
                <th class="w-16 text-center">Totaal</th>
              </tr>
            </thead>
            <tbody>
              {#each leaderboardQuery.data.mostCarded as player, i}
                <tr>
                  <td class="text-surface-400">{i + 1}</td>
                  <td
                    ><a
                      href="/players/{player.playerId}"
                      class="text-primary-500 hover:underline font-medium"
                      >{player.firstName} {player.lastName}</a
                    ></td
                  >
                  <td class="text-center text-surface-400"
                    >{player.shirtNumber}</td
                  >
                  <td class="text-center">{player.yellowCards}</td>
                  <td class="text-center">{player.redCards}</td>
                  <td class="text-center font-bold">{player.totalCards}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>
