import React, { useState } from 'react';
import { clipboard } from '@tauri-apps/api';
import WebSocket from 'isomorphic-ws';

import './index.css';
import './App.css';

const serverUrl: string = 'ws://127.0.0.1:9001';
const ws: WebSocket = new WebSocket(serverUrl);

function App() {
	const [selfIPEndPoint, setSelfIPEndPoint] = useState('localhost:0000');

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
					'(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?).){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?):[0-9]{5}'
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

			<button className='centered' onClick={sendClipboardDataToServer}>
				Send clipboard text to server
			</button>
		</div>
	);
}

function sendClipboardDataToServer() {
	clipboard.readText().then((value) => {
		ws.send(value);
		console.log(`Sent data: ${value}`);
	});
}

export default App;
