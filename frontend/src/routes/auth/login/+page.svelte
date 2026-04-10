<script lang="ts">
  import { login } from "$lib/api/auth";
  import { auth } from "$lib/state/auth.svelte";
  import { goto } from "$app/navigation";
  import GoogleOAuthButton from "$lib/components/GoogleOAuthButton.svelte";

  let email = $state("");
  let password = $state("");
  let error = $state("");

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = "";
    try {
      const res = await login({ email, password });
      auth.setUser(res.user);
      goto("/");
    } catch (err) {
      error = "Inloggen mislukt. Probeer het opnieuw.";
    }
  }
</script>

<div class="max-w-md mx-auto mt-8">
  <h1 class="text-2xl font-bold mb-6">Inloggen</h1>

  {#if error}
    <div class="card preset-tonal-error p-3 mb-4 text-sm">{error}</div>
  {/if}

  <form onsubmit={handleSubmit} class="space-y-4">
    <div>
      <label for="email" class="label-text">E-mail</label>
      <input
        id="email"
        type="email"
        bind:value={email}
        required
        class="input"
      />
    </div>
    <div>
      <label for="password" class="label-text">Wachtwoord</label>
      <input
        id="password"
        type="password"
        bind:value={password}
        required
        class="input"
      />
    </div>
    <button type="submit" class="btn w-full preset-filled-primary-500">
      Inloggen
    </button>
  </form>

  <div class="my-6 flex items-center gap-4">
    <hr class="flex-1 border-surface-300 dark:border-surface-700" />
    <span class="text-sm text-surface-400">of</span>
    <hr class="flex-1 border-surface-300 dark:border-surface-700" />
  </div>

  <GoogleOAuthButton />

  <p class="mt-6 text-center text-sm text-surface-400">
    Nog geen account? <a
      href="/auth/register"
      class="text-primary-500 font-medium hover:underline">Registreren</a
    >
  </p>
</div>
