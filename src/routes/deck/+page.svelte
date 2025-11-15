<script lang="ts">
	import * as types from '$lib/types';
	import { onMount } from 'svelte';

	import MatchStartSound from '$lib/assets/audio/match_start.wav';
	import AutoEndSound from '$lib/assets/audio/auto_end.wav';
	import PickUpControllersSound from '$lib/assets/audio/pick_up_controllers.wav';
	import ThreeTwoOneSound from '$lib/assets/audio/3-2-1.wav';
	import TeleopStartSound from '$lib/assets/audio/teleop_start.wav';
	import EndgameSound from '$lib/assets/audio/endgame.wav';
	import MatchEndSound from '$lib/assets/audio/match_end.wav';
	import AbortSound from '$lib/assets/audio/abort.wav';
	import RevealSound from '$lib/assets/audio/reveal.wav';
	import ResultsSound from '$lib/assets/audio/results.wav';
	import UnmuteSound from '$lib/assets/audio/unmute.wav';

	let ip = '';
	let eventCode = '';
	let useLocalTime = false;

	let dataType: string;
	let dataTs: number;
	let latestDataIndex = -1;

	let initialized = false;
	let enableAudio = false;

	let matchStartSound: HTMLAudioElement;
	let autoEndSound: HTMLAudioElement;
	let pickUpControllersSound: HTMLAudioElement;
	let threeTwoOneSound: HTMLAudioElement;
	let teleopStartSound: HTMLAudioElement;
	let endgameSound: HTMLAudioElement;
	let matchEndSound: HTMLAudioElement;
	let abortSound: HTMLAudioElement;
	let revealSound: HTMLAudioElement;
	let resultsSound: HTMLAudioElement;
	let unmuteSound: HTMLAudioElement;

	let matchStartVolume = 1;
	let autoEndVolume = 1;
	let pickUpControllersVolume = 1;
	let threeTwoOneVolume = 1;
	let teleopStartVolume = 1;
	let endgameVolume = 1;
	let matchEndVolume = 1;
	let abortVolume = 1;
	let revealVolume = 0.5;
	let resultsVolume = 1;
	let unmuteVolume = 1;

	let testSound: HTMLAudioElement;

	let timeSync = {
		id: 0,
		sentAt: 0,
		receivedTs: 0,
		receivedAtLocal: 0,
		interval: undefined as NodeJS.Timeout | undefined
	};
	let timer = 150;
	let timerPeriod = types.TimerPeriod.NONE;
	let timerRunning = false;
	let timeout: NodeJS.Timeout;

	function ts(): number {
		return performance.now() - timeSync.receivedAtLocal + timeSync.receivedTs;
	}

	function startTimer(seconds: number = 150) {
		let floored = Math.floor(seconds);
		timer = floored;

		if (seconds >= 150) {
			matchStartSound.pause();
			matchStartSound.currentTime = 0;
			matchStartSound.play();
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
						autoEndSound.pause();
						autoEndSound.currentTime = 0;
						autoEndSound.play();
					} else if (timer <= 1 && timerPeriod === types.TimerPeriod.TRANSITION) {
						timer = 120;
						timerPeriod = types.TimerPeriod.TELEOP;
						teleopStartSound.pause();
						teleopStartSound.currentTime = 0;
						teleopStartSound.play();
					} else if (timer <= 0) {
						timerRunning = false;
						timerPeriod = types.TimerPeriod.NONE;
					} else {
						timer--;
					}

					if (timerPeriod === types.TimerPeriod.AUTO && timer <= 130) {
					} else if (timerPeriod === types.TimerPeriod.TRANSITION) {
						if (timer === 6) {
							pickUpControllersSound.pause();
							pickUpControllersSound.currentTime = 0;
							pickUpControllersSound.play();
						} else if (timer === 3) {
							threeTwoOneSound.pause();
							threeTwoOneSound.currentTime = 0;
							threeTwoOneSound.play();
						}
					} else if (timerPeriod === types.TimerPeriod.TELEOP && timer > 0) {
						if (timer > 20) {
						} else {
							if (timer === 20) {
								endgameSound.pause();
								endgameSound.currentTime = 0;
								endgameSound.play();
							}
						}
					} else if (timer <= 0) {
						timerRunning = false;
						timerPeriod = types.TimerPeriod.NONE;
						matchEndSound.pause();
						matchEndSound.currentTime = 0;
						matchEndSound.play();
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
		abortSound.pause();
		abortSound.currentTime = 0;
		abortSound.play();
	}

	onMount(() => {
		const params = new URLSearchParams(window.location.search);
		ip = params.get('ip') || '';
		eventCode = params.get('eventCode') || '';
		useLocalTime = params.get('useLocalTime') === 'true';

		if (!ip || !eventCode) {
			window.location.href = `/`;
			return;
		}

		matchStartSound = document.getElementById('match-start-sound') as HTMLAudioElement;
		autoEndSound = document.getElementById('auto-end-sound') as HTMLAudioElement;
		pickUpControllersSound = document.getElementById(
			'pick-up-controllers-sound'
		) as HTMLAudioElement;
		threeTwoOneSound = document.getElementById('three-two-one-sound') as HTMLAudioElement;
		teleopStartSound = document.getElementById('teleop-start-sound') as HTMLAudioElement;
		endgameSound = document.getElementById('endgame-sound') as HTMLAudioElement;
		matchEndSound = document.getElementById('match-end-sound') as HTMLAudioElement;
		abortSound = document.getElementById('abort-sound') as HTMLAudioElement;
		revealSound = document.getElementById('reveal-sound') as HTMLAudioElement;
		resultsSound = document.getElementById('results-sound') as HTMLAudioElement;
		unmuteSound = document.getElementById('unmute-sound') as HTMLAudioElement;

		testSound = matchStartSound;

		connect();
	});

	function connect() {
		let ws = new WebSocket(`ws://${ip}/stream/display/command/?code=${eventCode}`);

		ws.onopen = () => {
			console.log('WebSocket connection opened');

			if (useLocalTime) {
				timeSync.receivedTs = Date.now();
				timeSync.receivedAtLocal = performance.now();

				if (!initialized) {
					initialized = true;
					updateType(dataType, dataTs);
				}
			} else {
				ws.send(`TIMESYNC:{"jsonrpc":"2.0","id":${timeSync.id},"method":"timesync"}`);
				timeSync.sentAt = ts();
				timeSync.interval = setInterval(() => {
					timeSync.id++;
					ws.send(`TIMESYNC:{"jsonrpc":"2.0","id":${timeSync.id},"method":"timesync"}`);
					timeSync.sentAt = ts();
				}, 30000);
			}
		};

		ws.onmessage = (event) => {
			if (event.data === 'ping') {
				ws.send('pong');
				return;
			}

			if (event.data === 'pong') {
				return;
			}

			if (event.data.startsWith('TIMESYNC:')) {
				const message = JSON.parse(event.data.substring(9));
				timeSync.receivedAtLocal = performance.now();
				timeSync.receivedTs = message.result + (ts() - timeSync.sentAt) / 2;

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

			if (['START_MATCH', 'ABORT_MATCH', 'SHOW_RESULTS'].includes(message.type)) {
				latestDataIndex = message.index;

				dataType = message.type;
				dataTs = message.ts;

				if (initialized) {
					updateType(message.type, message.ts);
				}
			}
		};

		ws.onclose = () => {
			console.log('WebSocket connection closed');
			clearInterval(timeSync.interval);
			connect();
		};

		ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};
	}

	function updateType(type: string, msgTs: number) {
		let offset = (ts() - msgTs) / 1000;

		switch (type) {
			case 'START_MATCH':
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
				}
				break;
			case 'SHOW_RESULTS':
				if (offset < 1000) {
					revealSound.play();
					setTimeout(() => {
						resultsSound.play();
					}, 7026);
				}
				break;
			case 'ABORT_MATCH':
				stopTimer();
				break;
		}
	}

	$: if (initialized) matchStartSound.volume = matchStartVolume;
	$: if (initialized) autoEndSound.volume = autoEndVolume;
	$: if (initialized) pickUpControllersSound.volume = pickUpControllersVolume;
	$: if (initialized) threeTwoOneSound.volume = threeTwoOneVolume;
	$: if (initialized) teleopStartSound.volume = teleopStartVolume;
	$: if (initialized) endgameSound.volume = endgameVolume;
	$: if (initialized) matchEndSound.volume = matchEndVolume;
	$: if (initialized) abortSound.volume = abortVolume;
	$: if (initialized) revealSound.volume = revealVolume;
	$: if (initialized) resultsSound.volume = resultsVolume;
	$: if (initialized) unmuteSound.volume = unmuteVolume;
</script>

<svelte:head>
	<title>{eventCode.toUpperCase()} | FTCLiveAuxDeck</title>
	<meta name="description" content="FTCLiveAuxDeck" />
</svelte:head>

<main>
	<div>
		<h3>FTCLiveAuxDeck</h3>
		<h5>{eventCode.toUpperCase()} | {ip}</h5>
		<h5>Timer: {Math.floor(timer / 60)}:{(timer % 60).toString().padStart(2, '0')}</h5>
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
			<select name="testSoundPicker" id="test-sound-picker" bind:value={testSound}>
				<option value={matchStartSound}>Match Start</option>
				<option value={autoEndSound}>Auto End</option>
				<option value={pickUpControllersSound}>Pick Up Controllers</option>
				<option value={threeTwoOneSound}>3-2-1</option>
				<option value={teleopStartSound}>Teleop Start</option>
				<option value={endgameSound}>Endgame</option>
				<option value={matchEndSound}>Match End</option>
				<option value={abortSound}>Abort</option>
				<option value={revealSound}>Reveal</option>
				<option value={resultsSound}>Results</option>
			</select>

			<button
				on:click={() => {
					testSound.play();
				}}>Play</button
			>

			<spacer></spacer>

			<button
				on:click={() => {
					unmuteSound.play();
				}}>Dung!</button
			>
		</test-sounds>
	</div>

	<audio-container>
		<audio id="match-start-sound" src={MatchStartSound} preload="auto"></audio>
		<audio id="auto-end-sound" src={AutoEndSound} preload="auto"></audio>
		<audio id="pick-up-controllers-sound" src={PickUpControllersSound} preload="auto"></audio>
		<audio id="three-two-one-sound" src={ThreeTwoOneSound} preload="auto"></audio>
		<audio id="teleop-start-sound" src={TeleopStartSound} preload="auto"></audio>
		<audio id="endgame-sound" src={EndgameSound} preload="auto"></audio>
		<audio id="match-end-sound" src={MatchEndSound} preload="auto"></audio>
		<audio id="abort-sound" src={AbortSound} preload="auto"></audio>
		<audio id="reveal-sound" src={RevealSound} preload="auto"></audio>
		<audio id="results-sound" src={ResultsSound} preload="auto"></audio>
		<audio id="unmute-sound" src={UnmuteSound} preload="auto"></audio>
	</audio-container>
</main>

{#if !enableAudio}
	<interact>
		<h1>Please click the button below to enable automatic audio playback.</h1>
		<button
			on:click={() => {
				enableAudio = true;
			}}>Click me!!!</button
		>
	</interact>
{/if}

<style>
	@import url('/style.css');

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

		audio-container {
			position: absolute;
			width: 0;
			height: 0;
			overflow: hidden;
			top: 100vh;
			opacity: 0;
			z-index: -1;
		}
	}

	interact {
		position: fixed;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		padding: 2rem;
		background-color: rgba(0, 0, 0, 0.8);
		color: white;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		gap: 1rem;
		z-index: 1000;

		h1 {
			text-align: center;
		}

		button {
			padding: 0.8rem 1.6rem;
			font-size: 1.2rem;
			background-color: #007bff;
			color: white;
			border: none;
			border-radius: 0.4rem;
			cursor: pointer;
		}
	}
</style>
