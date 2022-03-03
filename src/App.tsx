import React, { useState } from 'react';
import { clipboard } from '@tauri-apps/api';
import { appWindow, WebviewWindow } from '@tauri-apps/api/window';
import WebSocket from 'isomorphic-ws';

import './index.css';
import './App.css';

const serverUrl: string = 'ws://127.0.0.1:9001';
const ws: WebSocket = new WebSocket(serverUrl);

function App() {
	const [selfIPEndPoint, setSelfIPEndPoint] = useState('127.0.0.1:0000');

	initializeEventListeners();

	ws.onopen = async function open() {
		console.log(`Connected WebSocket to server ${serverUrl}`);

		// send_self_ip request
		ws.send('send_self_ip');
	};

	ws.onmessage = function incoming(data) {
		console.log(`Received data: ${data.data}`);

		if (
			data.data
				.toString()
				.match(
					'(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?):[0-9]{4}'
				)
		) {
			setSelfIPEndPoint(data.data.toString());
			console.log(
				`Caught self IP address message, data: ${data.data.toString()}`
			);
		}
	};

	return (
		<div>
			<h1>Your IP address is {selfIPEndPoint}.</h1>

			<p>
				Link device:
				<input
					className='centered'
					onChange={(e) => linkDevice(e.target.value)}
				/>
			</p>

			<button className='centered' onClick={sendClipboardDataToServer}>
				Send clipboard text to server
			</button>
		</div>
	);
}

async function initializeEventListeners() {
	// await appWindow.listen('local-ipendpoint-return', (event: { payload: any }) => {
	// 	// event.event is the event name (useful if you want to use a single callback fn for multiple event types)
	// 	// event.payload is the payload object
	// 	console.log(event.payload);
	// 	console.log('eee');
	// });
}

function sendClipboardDataToServer() {
	clipboard.readText().then((value) => {
		ws.send(`send_clipboard ${value}`);
		console.log(`Sent data: ${value}`);
	});
}

function linkDevice(ipEndPoint: string) {
	ws.send(`link_device ${ipEndPoint}`);
}

export default App;
