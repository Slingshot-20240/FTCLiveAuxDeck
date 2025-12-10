<script lang="ts">
  import * as types from "$lib/types";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    WebviewWindow,
    getAllWebviewWindows,
  } from "@tauri-apps/api/webviewWindow";

  let ip = "";
  let eventCode = "";
  let useLocalTime = false;

  let dataType: string;
  let dataTs: number;
  let latestDataIndex = -1;

  let firstTime = true;
  let initialized = false;

  let matchStartVolume = 1;
  let autoEndVolume = 1;
  let pickUpControllersVolume = 1;
  let threeTwoOneVolume = 1;
  let teleopStartVolume = 1;
  let endgameVolume = 1;
  let matchEndVolume = 1;
  let abortVolume = 1;
  let revealVolume = 0.6;
  let resultsVolume = 1;
  let unmuteVolume = 1;

  let testSound: string;

  let timeSync = {
    id: 0,
    sentAt: 0,
    receivedTs: 0,
    receivedAtLocal: 0,
    interval: undefined as any | undefined,
  };
  let timer = 150;
  let timerPeriod = types.TimerPeriod.NONE;
  let timerRunning = false;
  let timeout: any;

  function ts(): number {
    return performance.now() - timeSync.receivedAtLocal + timeSync.receivedTs;
  }

  function startTimer(seconds: number = 150) {
    let floored = Math.floor(seconds);
    timer = floored;

    if (150 - seconds <= 0.1) {
      playSound("match_start");
    }

    setTimeout(
      () => {
        timerRunning = true;
        let start = 0;
        let nextAt = 0;

        let interval = function () {
          if (!timerRunning) return;

          if (!start) {
            start = new Date().getTime();
            nextAt = start;
          } else if (timer <= 121 && timerPeriod === types.TimerPeriod.AUTO) {
            timer = 8;
            timerPeriod = types.TimerPeriod.TRANSITION;
            playSound("auto_end");
          } else if (
            timer <= 1 &&
            timerPeriod === types.TimerPeriod.TRANSITION
          ) {
            timer = 120;
            timerPeriod = types.TimerPeriod.TELEOP;
            playSound("teleop_start");
          } else if (timer <= 0) {
            timerRunning = false;
            timerPeriod = types.TimerPeriod.NONE;
          } else {
            timer--;
          }

          if (timerPeriod === types.TimerPeriod.AUTO && timer <= 130) {
          } else if (timerPeriod === types.TimerPeriod.TRANSITION) {
            if (timer === 6) {
              playSound("pick_up_controllers");
            } else if (timer === 3) {
              playSound("3-2-1");
            }
          } else if (timerPeriod === types.TimerPeriod.TELEOP && timer > 0) {
            if (timer > 20) {
            } else {
              if (timer === 20) {
                playSound("endgame");
              }
            }
          } else if (timer <= 0) {
            timerRunning = false;
            timerPeriod = types.TimerPeriod.NONE;
            playSound("match_end");
            return;
          }

          nextAt += 1000;

          timeout = setTimeout(interval, nextAt - new Date().getTime());
        };

        interval();
      },
      (seconds - floored) * 1000
    );
  }

  function stopTimer() {
    timer = 0;
    timerRunning = false;
    clearTimeout(timeout);
    playSound("abort");
  }

  async function playSound(soundName: string) {
    const volumeMap: Record<string, number> = {
      match_start: matchStartVolume,
      auto_end: autoEndVolume,
      pick_up_controllers: pickUpControllersVolume,
      "3-2-1": threeTwoOneVolume,
      teleop_start: teleopStartVolume,
      endgame: endgameVolume,
      match_end: matchEndVolume,
      abort: abortVolume,
      reveal: revealVolume,
      results: resultsVolume,
      unmute: unmuteVolume,
    };
    const volume = volumeMap[soundName] || 1;

    try {
      await invoke("play_sound", { soundName: soundName, volume: volume });
    } catch (error) {
      console.error("Failed to play sound:", error);
    }
  }

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    ip = params.get("ip") || "";
    eventCode = params.get("eventCode") || "";
    useLocalTime = params.get("useLocalTime") === "true";

    if (!ip || !eventCode) {
      window.location.href = `/`;
      return;
    }

    connect();
  });

  function connect() {
    let ws = new WebSocket(
      `ws://${ip}/stream/display/command/?code=${eventCode}`
    );

    ws.onopen = async () => {
      console.log("WebSocket connection opened");

      if (firstTime) {
        firstTime = false;
        try {
          await invoke("play_sound", { soundName: "unmute", volume: 1 });
        } catch (error) {
          console.error("Failed to play sound:", error);
        }
      }

      if (useLocalTime) {
        timeSync.receivedTs = Date.now();
        timeSync.receivedAtLocal = performance.now();

        if (!initialized) {
          initialized = true;
          updateType(dataType, dataTs);
        }
      } else {
        timeSync.sentAt = performance.now();
        ws.send(
          `TIMESYNC:{"jsonrpc":"2.0","id":${timeSync.id},"method":"timesync"}`
        );
        timeSync.interval = setInterval(() => {
          timeSync.id++;
          timeSync.sentAt = performance.now();
          ws.send(
            `TIMESYNC:{"jsonrpc":"2.0","id":${timeSync.id},"method":"timesync"}`
          );
        }, 30000);
      }
    };

    ws.onmessage = (event) => {
      if (event.data === "ping") {
        ws.send("pong");
        return;
      }

      if (event.data === "pong") {
        return;
      }

      if (event.data.startsWith("TIMESYNC:")) {
        const message = JSON.parse(event.data.substring(9));
        timeSync.receivedAtLocal = performance.now();
        timeSync.receivedTs =
          message.result + (timeSync.receivedAtLocal - timeSync.sentAt) / 2;

        if (!initialized) {
          initialized = true;
          updateType(dataType, dataTs);
        }
        return;
      }

      const message = JSON.parse(event.data);
      console.log(message);

      if (latestDataIndex >= message.index) {
        return;
      }

      if (
        ["START_MATCH", "ABORT_MATCH", "SHOW_RESULTS"].includes(message.type)
      ) {
        latestDataIndex = message.index;

        dataType = message.type;
        dataTs = message.ts;

        if (initialized) {
          updateType(message.type, message.ts);
        }
      }
    };

    ws.onclose = () => {
      console.log("WebSocket connection closed");
      initialized = false;
      clearInterval(timeSync.interval);
      connect();
    };

    ws.onerror = (error) => {
      console.error("WebSocket error:", error);
    };
  }

  function updateType(type: string, msgTs: number) {
    let offset = (ts() - msgTs) / 1000;

    switch (type) {
      case "START_MATCH":
        let startSeconds = 30 - offset;

        if (startSeconds < 0) {
          if (startSeconds > -8) {
            timerPeriod = types.TimerPeriod.TRANSITION;
            startTimer(Math.abs(startSeconds));
          } else {
            startSeconds = 158 - offset;
            if (startSeconds > 0) {
              timerPeriod = types.TimerPeriod.TELEOP;
              startTimer(startSeconds);
            }
          }
        } else {
          timerPeriod = types.TimerPeriod.AUTO;
          startTimer(150 - offset);
          console.log(offset);
        }
        break;
      case "SHOW_RESULTS":
        if (offset > 0.1) {
          return;
        }

        playSound("reveal");
        setTimeout(() => {
          playSound("results");
        }, 7026);
        break;
      case "ABORT_MATCH":
        if (offset > 0.1) {
          return;
        }

        stopTimer();
        break;
    }
  }

  async function home() {
    const existing = (await getAllWebviewWindows()).find(
      (w) => w.label === "main"
    );

    if (existing) {
      existing.setFocus();
      return;
    }

    const mainWindow = new WebviewWindow("main", {
      title: "FTCLiveAuxDeck",
      width: 400,
      height: 300,
      resizable: false,
    });
    mainWindow.setFocus();
  }

  onMount(async () => {
    document.addEventListener("keydown", (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "n") {
        home();
      }
    });
  });
</script>

<svelte:head>
  <title>{eventCode.toUpperCase()} | FTCLiveAuxDeck</title>
  <meta name="description" content="FTCLiveAuxDeck" />
</svelte:head>

<main>
  <div>
    <h3>FTCLiveAuxDeck</h3>
    <h5>{eventCode.toUpperCase()} | {ip}</h5>
    <h5>
      Timer: {Math.floor(timer / 60)}:{(timer % 60).toString().padStart(2, "0")}
    </h5>
  </div>

  <div>
    <h4>Volumes</h4>

    <volume-container>
      <div class="volume-block">
        <label for="match-start-volume">Match Start</label>
        <input
          type="number"
          name="matchStartVolume"
          id="match-start-volume"
          bind:value={matchStartVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="auto-end-volume">Auto End</label>
        <input
          type="number"
          name="autoEndVolume"
          id="auto-end-volume"
          bind:value={autoEndVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="pick-up-controllers-volume">Controllers</label>
        <input
          type="number"
          name="pickUpControllersVolume"
          id="pick-up-controllers-volume"
          bind:value={pickUpControllersVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="three-two-one-volume">3-2-1</label>
        <input
          type="number"
          name="threeTwoOneVolume"
          id="three-two-one-volume"
          bind:value={threeTwoOneVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="teleop-start-volume">Teleop Start</label>
        <input
          type="number"
          name="teleopStartVolume"
          id="teleop-start-volume"
          bind:value={teleopStartVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="endgame-volume">Endgame</label>
        <input
          type="number"
          name="endgameVolume"
          id="endgame-volume"
          bind:value={endgameVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="match-end-volume">Match End</label>
        <input
          type="number"
          name="matchEndVolume"
          id="match-end-volume"
          bind:value={matchEndVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="abort-volume">Abort</label>
        <input
          type="number"
          name="abortVolume"
          id="abort-volume"
          bind:value={abortVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="reveal-volume">Reveal</label>
        <input
          type="number"
          name="revealVolume"
          id="reveal-volume"
          bind:value={revealVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="results-volume">Results</label>
        <input
          type="number"
          name="resultsVolume"
          id="results-volume"
          bind:value={resultsVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>

      <div class="volume-block">
        <label for="unmute-volume">Dung!</label>
        <input
          type="number"
          name="unmuteVolume"
          id="unmute-volume"
          bind:value={unmuteVolume}
          min="0"
          max="1"
          step="0.05"
        />
      </div>
    </volume-container>
  </div>

  <div>
    <h4>Test Sounds</h4>
    <test-sounds>
      <select
        name="testSoundPicker"
        id="test-sound-picker"
        bind:value={testSound}
      >
        <option value="match_start">Match Start</option>
        <option value="auto_end">Auto End</option>
        <option value="pick_up_controllers">Pick Up Controllers</option>
        <option value="3-2-1">3-2-1</option>
        <option value="teleop_start">Teleop Start</option>
        <option value="endgame">Endgame</option>
        <option value="match_end">Match End</option>
        <option value="abort">Abort</option>
        <option value="reveal">Reveal</option>
        <option value="results">Results</option>
      </select>

      <button
        on:click={() => {
          playSound(testSound);
        }}>Play</button
      >

      <spacer></spacer>

      <button
        on:click={() => {
          playSound("unmute");
        }}>Dung!</button
      >
    </test-sounds>
  </div>
</main>

<style>
  @import url("/style.css");

  main {
    padding: 0.8rem;
    gap: 0.8rem;

    div {
      display: flex;
      flex-direction: column;
      gap: 0.2rem;
    }

    volume-container {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 0.4rem;

      .volume-block {
        display: grid;
        grid-template-columns: 4fr 1fr;
        padding: 0.2rem;
        border: 0.1rem solid #aaa;

        input[type="number"] {
          width: 2.4rem;
        }
      }
    }

    test-sounds {
      display: flex;
      align-items: center;
      gap: 0.4rem;

      spacer {
        flex-grow: 1;
      }
    }
  }
</style>
