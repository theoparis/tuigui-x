use std::io;
use tuigui::{Backend, ClearType};
use tuigui::{Position, Size};
use xcb::x;

// This is cursed
pub struct XBackend {
	connection: xcb::Connection,
	window: x::Window,
	cursor_position: Position,
}

impl XBackend {
	pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
		let (connection, screen_number) = xcb::Connection::connect(None)?;

		let setup = connection.get_setup();
		let screen = setup.roots().nth(screen_number as usize).unwrap();

		let window: x::Window = connection.generate_id();
		connection.send_request(&x::CreateWindow {
			depth: x::COPY_FROM_PARENT as u8,
			wid: window,
			parent: screen.root(),
			x: 0,
			y: 0,
			width: 150,
			height: 150,
			border_width: 10,
			class: x::WindowClass::InputOutput,
			visual: screen.root_visual(),
			value_list: &[
				x::Cw::BackPixel(screen.white_pixel()),
				x::Cw::EventMask(
					x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS,
				),
			],
		});

		connection.send_request(&x::MapWindow { window });

		let gc: x::Gcontext = connection.generate_id();

		connection.send_request(&x::CreateGc {
			cid: gc,
			drawable: x::Drawable::Window(window),
			value_list: &[
				x::Gc::Foreground(screen.black_pixel()),
				x::Gc::GraphicsExposures(false),
			],
		});

		connection.send_request(&x::FreeGc { gc });

		connection.flush()?;

		let event = match connection.wait_for_event() {
			Err(err) => {
				panic!("unexpected error: {:#?}", err);
			}
			Ok(event) => event,
		};
		match event {
			xcb::Event::X(x::Event::Expose(_ev)) => Ok(Self {
				connection,
				window,
				cursor_position: Position::new(0, 0),
			}),
			_ => Err("Uh oh".into()),
		}
	}
}

impl Backend for XBackend {
	fn flush(&mut self) -> Result<(), io::Error> {
		self.connection.flush().unwrap();
		Ok(())
	}

	fn terminal_size(&self) -> Result<Size, io::Error> {
		let cookie = self.connection.send_request(&x::GetGeometry {
			drawable: x::Drawable::Window(self.window),
		});

		let reply = self.connection.wait_for_reply(cookie).unwrap();

		Ok(Size::new(reply.width() / 8, reply.height() / 16))
	}

	fn set_cursor_pos(&mut self, position: Position) -> Result<(), io::Error> {
		self.cursor_position = position;

		Ok(())
	}

	fn alt_screen(&mut self, enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn raw_mode(&mut self, enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn clear(&mut self, clear_type: ClearType) -> Result<(), io::Error> {
		let size = self.terminal_size()?;
		let width = size.cols * 8;
		let height = size.rows * 16;

		match clear_type {
			ClearType::All => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: 0,
					y: 0,
					width,
					height,
				});
			}
			ClearType::FromCursorDown => {
				let x = self.cursor_position.col * 8;
				let y = self.cursor_position.row * 16;

				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x,
					y,
					width: width - x as u16,
					height: height - y as u16,
				});
			}
			ClearType::FromCursorUp => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: 0,
					y: 0,
					width,
					height: self.cursor_position.row as u16 * 16,
				});
			}
			ClearType::Purge => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: 0,
					y: 0,
					width,
					height,
				});
			}
			ClearType::CurrentLine => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: 0,
					y: self.cursor_position.row * 16,
					width,
					height: 16,
				});
			}
			ClearType::UntilNewLine => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: self.cursor_position.col * 8,
					y: self.cursor_position.row * 16,
					width: width - self.cursor_position.col as u16 * 8,
					height: 16,
				});
			}
		}

		Ok(())
	}

	fn show_cursor(&mut self, enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn print<S: AsRef<str>>(&mut self, content: S) -> Result<(), io::Error> {
		let drawable = x::Drawable::Window(self.window);

		let content = content.as_ref();
		let (color, content) = decode_ansi(content);

		dbg!(&content);

		for c in content.chars() {
			if c == '\n' {
				self.cursor_position.row += 1;
				self.cursor_position.col = 0;

				continue;
			}

			let x = self.cursor_position.col * 8;
			let y = self.cursor_position.row * 16;

			let gc: x::Gcontext = self.connection.generate_id();

			if c == ' ' {
				self.cursor_position.col += 1;
				continue;
			}

			if c == '█' {
				self.connection.send_request(&x::CreateGc {
					cid: gc,
					drawable,
					value_list: &[x::Gc::Foreground(color)],
				});

				self.connection.send_request(&x::PolyFillRectangle {
					drawable,
					gc,
					rectangles: &[x::Rectangle {
						x,
						y,
						width: 8,
						height: 16,
					}],
				});
			} else if c == ' ' {
				self.connection.send_request(&x::PolyFillRectangle {
					drawable,
					gc,
					rectangles: &[x::Rectangle {
						x,
						y,
						width: 8,
						height: 16,
					}],
				});
			} else {
				self.connection.send_request(&x::CreateGc {
					cid: gc,
					drawable,
					value_list: &[x::Gc::Foreground(color)],
				});

				self.connection.send_request(&x::PolyFillRectangle {
					drawable,
					gc,
					rectangles: &[x::Rectangle {
						x,
						y,
						width: 8,
						height: 16,
					}],
				});

				self.connection.send_request(&x::ImageText8 {
					drawable,
					gc,
					x,
					y: y + 16,
					string: &[c as u8],
				});
			}

			self.connection.send_request(&x::FreeGc { gc });

			self.cursor_position.col += 1;
		}

		Ok(())
	}

	fn cursor_position(&self) -> Result<Position, io::Error> {
		Ok(self.cursor_position)
	}
}

fn decode_ansi(content: &str) -> (u32, String) {
	let mut final_color = 0x000000;

	let content = content.to_string();
	let mut content = content.chars();

	let mut new_content = String::new();

	while let Some(c) = content.next() {
		if c == '\x1b' {
			let c = content.next().unwrap();

			if c == '[' {
				let mut params = String::new();

				loop {
					let c = content.next().unwrap();

					if c == 'm' {
						break;
					}

					params.push(c);
				}

				let params: Vec<&str> = params.split(';').collect();

				for param in params {
					let param = param.parse::<u32>().unwrap();

					if param == 0 {
						final_color = 0x000000;
					} else if param == 30 {
						final_color = 0x000000;
					} else if param == 31 {
						final_color = 0x800000;
					} else if param == 32 {
						final_color = 0x008000;
					} else if param == 33 {
						final_color = 0x808000;
					} else if param == 34 {
						final_color = 0x000080;
					} else if param == 35 {
						final_color = 0x800080;
					} else if param == 36 {
						final_color = 0x008080;
					} else if param == 37 {
						final_color = 0xc0c0c0;
					} else if param == 40 {
						final_color = 0x000000;
					} else if param == 41 {
						final_color = 0x800000;
					} else if param == 42 {
						final_color = 0x008000;
					} else if param == 43 {
						final_color = 0x808000;
					} else if param == 44 {
						final_color = 0x000080;
					} else if param == 45 {
						final_color = 0x800080;
					} else if param == 46 {
						final_color = 0x008080;
					} else if param == 47 {
						final_color = 0xc0c0c0;
					}
				}
			}
		}

		new_content.push(c);
	}

	// new_content =  "\u{1b}█\u{1b}"
	let new_content = new_content.replace("\u{1b}", "");

	(final_color, new_content)
}
