<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let ip = "";
  let eventCode = "";
  let useLocalTime = false;

  function nav() {
    window.location.href = `/deck?ip=${ip}&eventCode=${eventCode.toLowerCase()}&useLocalTime=${useLocalTime}`;
  }

  (async () => {
    try {
      await invoke("play_sound", { soundName: "unmute", volume: 1 });
    } catch (error) {
      console.error("Failed to play sound:", error);
    }
  })();
</script>

<svelte:head>
  <title>FTCLiveAuxDeck</title>
  <meta name="description" content="FTCLiveAuxDeck" />
</svelte:head>

<main>
  <h1>FTCLiveAuxDeck</h1>
  <form on:submit|preventDefault={nav}>
    <input
      type="text"
      name="ip"
      id="ip"
      bind:value={ip}
      placeholder="FTCLive Host IP Address"
    />
    <input
      type="text"
      name="eventCode"
      id="eventCode"
      bind:value={eventCode}
      placeholder="Event Code"
    />
    <div>
      <input
        type="checkbox"
        name="useLocalTime"
        id="use-local-time"
        bind:checked={useLocalTime}
      />
      <label for="use-local-time">Use Local Time</label>
    </div>

    {#if ip && eventCode}
      <button on:click={nav}>Go →</button>
    {/if}
  </form>
</main>

<style>
  @import url("/style.css");

  main {
    align-items: center;
    gap: 0.8rem;
  }

  input[type="text"] {
    padding: 0.4rem;
    font-size: 1rem;
    width: 100%;
    max-width: 20rem;
    border: none;
    border-radius: 0.4rem;
  }

  form,
  div {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  form {
    flex-direction: column;
  }

  button {
    padding: 0.4rem 0.8rem;
    font-size: 1rem;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 0.4rem;
    cursor: pointer;
  }
</style>
