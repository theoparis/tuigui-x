use std::io;
use tuigui::{
	AnsiColor, Backend, ClearType, ContentProcessorOutput as _, StyleGround,
};
use tuigui::{Position, Size};
use xcb::x;

pub struct Printable {
	pub value: char,
	pub color: u32,
}

impl tuigui::ContentProcessorOutput for Printable {
	fn clear_output() -> Self {
		Self {
			value: ' ',
			color: 0,
		}
	}
}

pub struct XContentProcessor {}

impl tuigui::ContentProcessor<Printable> for XContentProcessor {
	fn process(&mut self, character: char, style: &tuigui::Style) -> Printable {
		if let StyleGround::Color(color) = style.fg {
			return Printable {
				value: character,
				color: match color {
					tuigui::Color::Custom { r, g, b } => {
						(r as u32) << 16 | (g as u32) << 8 | b as u32
					}
					tuigui::Color::Ansi(ansi_color) => match ansi_color {
						AnsiColor::Black => 0x000000,
						AnsiColor::Red => 0xFF0000,
						AnsiColor::Green => 0x00FF00,
						AnsiColor::Yellow => 0xFFFF00,
						AnsiColor::Blue => 0x0000FF,
						AnsiColor::Magenta => 0xFF00FF,
						AnsiColor::Cyan => 0x00FFFF,
						AnsiColor::White => 0xFFFFFF,
						AnsiColor::BrightBlack => 0x808080,
						AnsiColor::BrightRed => 0xFF8080,
						AnsiColor::BrightGreen => 0x80FF80,
						AnsiColor::BrightYellow => 0xFFFF80,
						AnsiColor::BrightBlue => 0x8080FF,
						AnsiColor::BrightMagenta => 0xFF80FF,
						AnsiColor::BrightCyan => 0x80FFFF,
						AnsiColor::BrightWhite => 0xC0C0C0,
					},
				},
			};
		}

		Printable::clear_output()
	}
}

const DEFAULT_WIDTH: u16 = 640;
const DEFAULT_HEIGHT: u16 = 480;
const FONT_WIDTH: u16 = 8;
const FONT_HEIGHT: u16 = 16;
const FONT_OFFSET_X: i16 = 2;
const FONT_OFFSET_Y: i16 = 3;
const BORDER_WIDTH: u16 = 10;

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
			width: DEFAULT_WIDTH,
			height: DEFAULT_HEIGHT,
			border_width: BORDER_WIDTH,
			class: x::WindowClass::InputOutput,
			visual: screen.root_visual(),
			value_list: &[
				x::Cw::BackPixel(screen.black_pixel()),
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

impl Backend<Printable> for XBackend {
	fn flush(&mut self) -> Result<(), io::Error> {
		self.connection.flush().unwrap();
		Ok(())
	}

	fn terminal_size(&self) -> Result<Size, io::Error> {
		let cookie = self.connection.send_request(&x::GetGeometry {
			drawable: x::Drawable::Window(self.window),
		});

		let reply = self.connection.wait_for_reply(cookie).unwrap();

		Ok(Size::new(
			reply.width() / FONT_WIDTH,
			reply.height() / FONT_HEIGHT,
		))
	}

	fn set_cursor_pos(&mut self, position: Position) -> Result<(), io::Error> {
		self.cursor_position = position;

		Ok(())
	}

	fn alt_screen(&mut self, _enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn raw_mode(&mut self, _enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn clear(&mut self, clear_type: ClearType) -> Result<(), io::Error> {
		let size = self.terminal_size()?;
		let width = size.cols * FONT_WIDTH;
		let height = size.rows * FONT_HEIGHT;

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
				let x = self.cursor_position.col * FONT_WIDTH as i16;
				let y = self.cursor_position.row * FONT_HEIGHT as i16;

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
					height: self.cursor_position.row as u16 * FONT_HEIGHT,
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
					y: self.cursor_position.row * FONT_HEIGHT as i16,
					width,
					height: FONT_HEIGHT,
				});
			}
			ClearType::UntilNewLine => {
				self.connection.send_request(&x::ClearArea {
					exposures: false,
					window: self.window,
					x: self.cursor_position.col * FONT_WIDTH as i16,
					y: self.cursor_position.row * FONT_HEIGHT as i16,
					width: width - self.cursor_position.col as u16 * FONT_WIDTH,
					height: FONT_HEIGHT,
				});
			}
		}

		Ok(())
	}

	fn show_cursor(&mut self, _enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn print(&mut self, content: Printable) -> Result<(), io::Error> {
		let drawable = x::Drawable::Window(self.window);

		let color = content.color;
		let content = content.value;

		let x = self.cursor_position.col * FONT_WIDTH as i16;
		let y = self.cursor_position.row * FONT_HEIGHT as i16;

		let gc: x::Gcontext = self.connection.generate_id();

		self.connection.send_request(&x::CreateGc {
			cid: gc,
			drawable,
			value_list: &[x::Gc::Foreground(color)],
		});

		if content == '█' {
			self.connection.send_request(&x::PolyFillRectangle {
				drawable,
				gc,
				rectangles: &[x::Rectangle {
					x,
					y,
					width: FONT_WIDTH,
					height: FONT_HEIGHT,
				}],
			});
		} else {
			self.connection.send_request(&x::ClearArea {
				window: self.window,
				exposures: false,
				x,
				y,
				width: FONT_WIDTH,
				height: FONT_HEIGHT,
			});

			self.connection.send_request(&x::ImageText8 {
				drawable,
				gc,
				x: x + FONT_OFFSET_X,
				y: y + (FONT_HEIGHT as i16) - FONT_OFFSET_Y,
				string: &[content as u8, ' ' as u8],
			});
		}

		self.connection.send_request(&x::FreeGc { gc });

		self.cursor_position.col += 1;

		Ok(())
	}

	fn cursor_position(&self) -> Result<Position, io::Error> {
		Ok(self.cursor_position)
	}

	fn capture_mouse(&mut self, _enable: bool) -> Result<(), io::Error> {
		Ok(())
	}

	fn begin_sync_update(&mut self) -> Result<(), io::Error> {
		Ok(())
	}

	fn end_sync_update(&mut self) -> Result<(), io::Error> {
		Ok(())
	}
}
