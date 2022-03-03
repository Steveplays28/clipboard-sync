#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

mod cmd;

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::thread::spawn;
use tauri::{
	api::dialog::ask, http::ResponseBuilder, CustomMenuItem, Manager, Menu, MenuItem, RunEvent,
	Submenu, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder,
};
use tauri_plugin_store;
use tungstenite::{accept, connect, Message};

#[derive(Serialize)]
struct Reply {
	data: String,
}

#[derive(Serialize, Deserialize)]
struct HttpPost {
	foo: String,
	bar: String,
}

#[derive(Serialize)]
struct HttpReply {
	msg: String,
	request: HttpPost,
}

// The payload type must implement Serialize
// For global events, it also must implement Clone
#[derive(Clone, serde::Serialize)]
struct Payload {
	message: String,
}

#[tauri::command]
async fn menu_toggle(window: tauri::Window) {
	window.menu_handle().toggle().unwrap();
}

fn main() {
	spawn(|| {
		let default_socket_addr: SocketAddr =
			SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001);
		let mut local_addr: SocketAddr = default_socket_addr;

		let server = TcpListener::bind(default_socket_addr).unwrap();

		for stream in server.incoming() {
			spawn(move || {
				local_addr = stream
					.as_ref()
					.unwrap()
					.local_addr()
					.unwrap_or(default_socket_addr);
				let mut websocket = accept(stream.unwrap()).unwrap();

				loop {
					let msg = websocket.read_message().unwrap();

					// TODO: Store linked devices using IPs
					// [ ] Get IP address of device using `stream.unwrap().peer_addr();`
					// [ ] Store IP address of device using Tauri

					// Send back IP address of client
					if msg.to_text().unwrap_or("unwrap_error") == "send_self_ip" {
						websocket.write_message(Message::text(local_addr.to_string()));
					}

					// Send link_device request to other device
					if msg
						.to_text()
						.unwrap_or("unwrap_error")
						.starts_with("link_device")
					{
						let otherDevice = connect(
							"ws://".to_owned()
								+ msg
									.to_text()
									.unwrap_or("unwrap_error")
									.split(" ")
									.nth(0)
									.unwrap_or(""),
						);
						otherDevice.unwrap().0.write_message(Message::text("helo"));
					}

					// We do not want to send back ping/pong messages.
					// if msg.is_binary() || msg.is_text() {
					// 	// Send same message back to client
					// 	websocket.write_message(msg).unwrap();
					// }
				}
			});
		}
	});

	// System tray with menu buttons
	let device_button =
		CustomMenuItem::new("device_1".to_string(), "Copy clipboard data from device 1");
	let quit = CustomMenuItem::new("quit".to_string(), "Quit");
	let system_tray_menu = SystemTrayMenu::new()
		.add_item(device_button)
		.add_native_item(SystemTrayMenuItem::Separator)
		.add_item(quit);
	let system_tray = SystemTray::new().with_menu(system_tray_menu);

	#[allow(unused_mut)]
	let mut app = tauri::Builder::default()
		.plugin(tauri_plugin_store::PluginBuilder::default().build())
		.on_page_load(|window, _| {
			window.listen("link-device", move |event| {
				window.emit(
					"link-device-success",
					Payload {
						message: stream
							.as_ref()
							.unwrap()
							.local_addr()
							.unwrap_or(default_socket_addr)
							.into(),
					},
				);
			});

			Ok(());
		})
		.system_tray(system_tray)
		.on_system_tray_event(|app, event| match event {
			SystemTrayEvent::MenuItemClick { id, .. } => {
				let _item_handle = app.tray_handle().get_item(&id);

				match id.as_str() {
					"quit" => {
						std::process::exit(0);
					}
					_ => {}
				}
			}
			SystemTrayEvent::LeftClick { .. } => {
				app.create_window("main", tauri::WindowUrl::default(), |win, webview| {
					let win = win
						.title("Clipboard Sync")
						.resizable(true)
						.inner_size(800.0, 550.0)
						.min_inner_size(400.0, 200.0);
					return (win, webview);
				});
			}
			_ => {}
		})
		.invoke_handler(tauri::generate_handler![
			cmd::hello_world_test,
			cmd::ls_test,
			menu_toggle,
		])
		.build(tauri::generate_context!())
		.expect("error while building tauri application");

	#[cfg(target_os = "macos")]
	app.set_activation_policy(tauri::ActivationPolicy::Regular);

	app.run(|app_handle, e| match e {
		// On application ready (triggered only once)
		RunEvent::Ready => {}

		// Triggered when a window is trying to close
		RunEvent::CloseRequested { label, api, .. } => {
			let app_handle = app_handle.clone();
			let window = app_handle.get_window(&label).unwrap();

			// Prevent the window from closing
			api.prevent_close();

			// Ask the user if they want to quit
			ask(
				Some(&window),
				"Close window",
				"Are you sure that you want to close this window?",
				move |answer| {
					if answer {
						// .close() cannot be called on the main thread
						std::thread::spawn(move || {
							app_handle.get_window(&label).unwrap().close().unwrap();
						});
					}
				},
			);
		}

		// Keep the event loop running even if all windows are closed
		// This allow us to catch system tray events when there is no window
		RunEvent::ExitRequested { api, .. } => {
			api.prevent_exit();
		}
		_ => {}
	});
}

pub fn get_menu() -> Menu {
	#[allow(unused_mut)]
	let mut disable_item =
		CustomMenuItem::new("disable-menu", "Disable menu").accelerator("CmdOrControl+D");
	#[allow(unused_mut)]
	let mut test_item = CustomMenuItem::new("test", "Test").accelerator("CmdOrControl+T");
	#[cfg(target_os = "macos")]
	{
		disable_item = disable_item.native_image(tauri::NativeImage::MenuOnState);
		test_item = test_item.native_image(tauri::NativeImage::Add);
	}

	// create a submenu
	let my_sub_menu = Menu::new().add_item(disable_item);

	let my_app_menu = Menu::new()
		.add_native_item(MenuItem::Copy)
		.add_submenu(Submenu::new("Sub menu", my_sub_menu));

	let test_menu = Menu::new()
		.add_item(CustomMenuItem::new(
			"selected/disabled",
			"Selected and disabled",
		))
		.add_native_item(MenuItem::Separator)
		.add_item(test_item);

	// add all our childs to the menu (order is how they'll appear)
	Menu::new()
		.add_submenu(Submenu::new("My app", my_app_menu))
		.add_submenu(Submenu::new("Other menu", test_menu))
}
