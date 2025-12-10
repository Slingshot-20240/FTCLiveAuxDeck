<script lang="ts">
  import {
    WebviewWindow,
    getAllWebviewWindows,
  } from "@tauri-apps/api/webviewWindow";

  let ip = "";
  let eventCode = "";
  let useLocalTime = false;

  async function nav() {
    const existing = (await getAllWebviewWindows()).find(
      (w) => w.label === "deck-" + eventCode
    );

    if (existing) {
      existing.setFocus();
      return;
    }

    new WebviewWindow("deck-" + eventCode, {
      title: eventCode.toUpperCase(),
      url: `/deck?ip=${ip}&eventCode=${eventCode.toLowerCase()}&useLocalTime=${useLocalTime}`,
      width: 680,
      height: 280,
      minWidth: 680,
      minHeight: 280,
    });
  }
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
      <button>Go →</button>
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
