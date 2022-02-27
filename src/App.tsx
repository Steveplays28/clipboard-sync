import React, { useState } from 'react';
import { clipboard } from '@tauri-apps/api';
import WebSocket from 'isomorphic-ws';

import './index.css';
import './App.css';

const serverUrl: string = 'ws://127.0.0.1:9001';
const ws: WebSocket = new WebSocket(serverUrl);

function App() {
	ws.onopen = async function open() {
		console.log(`Connected WebSocket to server ${serverUrl}`);
	};

	ws.onmessage = function incoming(data) {
		console.log(`Received data: ${data.data}`);
	};

	const components = ['button', 'button', 'button'];
	return (
		<div>
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
