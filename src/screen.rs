use image::ImageBuffer;
use serialport::SerialPort;

pub const WIDTH: u16 = 320;
pub const HEIGHT: u16 = 480;
const SCREEN_SERIAL: &str = "USB35INCHIPSV2";

#[allow(dead_code)]
pub enum Orientation {
	Portrait = 0,
	ReversePortrait = 1,
	Landscape = 2,
	ReverseLandscape = 3,
}

#[allow(dead_code)]
pub enum ScreenCommand {
	Reset = 101,
	Clear = 102,
	ToBlack = 103,
	ScreenOff = 108,
	ScreenOn = 109,
	SetBrigthness = 110,
	SetOrientation = 121,
	DisplayBitmap = 197,
}

pub struct Screen {
	port: Box<dyn SerialPort>
}

impl Screen {
	/// Automagically finds the port the screen is connected to.
	/// If the screen is not found, an empty string is returned.
	/// If the screen is found, the port name is returned.
	pub fn find_port() -> Result<String, serialport::Error> {
		let ports = serialport::available_ports()?;
		let mut port = "".to_string();
		for p in ports {
			match p.port_type {
				serialport::SerialPortType::UsbPort(info) => {
					let sn = info.serial_number.unwrap_or("--".to_string());
					if sn == SCREEN_SERIAL {
						port = p.port_name.clone();
					}
				}
				_ => { /* ignore */ }
			}
		}

		Ok(port)
	}

	/// Creates a new Screen instance.
	/// Requires the port name as a parameter. It can be obtained by calling the `find_port()` function.
	pub fn new(port_name: String) -> Result<Screen, serialport::Error> {
		let port = serialport::new(&port_name, 115_200)
			.timeout(std::time::Duration::from_secs(1))
			.open()?;
		Ok(Screen { port })
	}
}

impl Screen {
	fn send_command(&mut self, x: u16, y: u16, ex: u16, ey: u16, cmd: ScreenCommand) -> Result<(), crate::errors::ScreenError> {
		let mut byte_buffer = [0u8; 6];
		byte_buffer[0] = (x >> 2) as u8;
		byte_buffer[1] = (((x & 3) << 6) + (y >> 4)) as u8;
		byte_buffer[2] = (((y & 15) << 4) + (ex >> 6)) as u8;
		byte_buffer[3] = (((ex & 63) << 2) + (ey >> 8)) as u8;
		byte_buffer[4] = (ey & 255) as u8;
		byte_buffer[5] = cmd as u8;

		self.port.write(&byte_buffer).map_err(|_| crate::errors::ScreenError::WriteError)?;

		Ok(())
	}

	/// Sets the screens orientation
	#[allow(unused)]
	pub fn orientation(&mut self, orientation: Orientation) -> Result<(), crate::errors::ScreenError> {
		let bytes = vec![0, 0, 0, 0, 0, ScreenCommand::SetOrientation as u8, (orientation as u8) + 100, 3, 200, 4, 0];

		self.port.write(&bytes).map_err(|_| crate::errors::ScreenError::WriteError)?;

		Ok(())
	}

	#[allow(unused)]
	/// Clears the screen to white.
	/// Does not work correctly in landscape mode, switch to Portrait mode before using this function.
	pub fn clear(&mut self) -> Result<(), crate::errors::ScreenError> {
		self.send_command(0, 0, 0, 0, ScreenCommand::Clear)
	}

	#[allow(unused)]
	/// Clears the screen to black.
	/// Does not work correctly in landscape mode, switch to Portrait mode before using this function.
	pub fn to_black(&mut self) -> Result<(), crate::errors::ScreenError> {
		self.send_command(0, 0, 0, 0, ScreenCommand::ToBlack)
	}

	#[allow(unused)]
	/// Sets the brightness of the screen.
	/// Level must be between 0 and 255. 0 is the brightest, 255 is the darkest.
	pub fn brightness(&mut self, level: u8) -> Result<(), crate::errors::ScreenError> {
		let level = 255 - level; // Invert level
		self.send_command(level as u16, 0, 0, 0, ScreenCommand::SetBrigthness)
	}

	#[allow(unused)]
	/// Turns the screen off. It will still be powered on, but the screen will be black.
	/// To turn the screen back on, use the screen_on() function. Retains the current image.
	pub fn screen_off(&mut self) -> Result<(), crate::errors::ScreenError> {
		self.send_command(0, 0, 0, 0, ScreenCommand::ScreenOff)
	}

	#[allow(unused)]
	/// Turns the screen on. The screen will display the last image that was drawn.
	pub fn screen_on(&mut self) -> Result<(), crate::errors::ScreenError> {
		self.send_command(0, 0, 0, 0, ScreenCommand::ScreenOn)
	}

	// // Not working
	// #[allow(unused)]
	// pub fn reset(&mut self) {
	// 	self.send_command(0, 0, 0, 0, ScreenCommand::RESET);

	// 	std::thread::sleep(std::time::Duration::from_secs(3));

	// 	// Reconnect to the screen
	// 	// self.port = serialport::new(&self.port_name, 115_200)
	// 	// 	.timeout(std::time::Duration::from_secs(1))
	// 	// 	.open()
	// 	// 	.expect("Failed to open port");
	// }

	#[allow(unused)]
	/// Draws an `ImageBuffer` to the screen.
	/// The image must be 320x480 or 480x320. Although not checked, the orientation of the image should match the orientation of the screen.
	/// Otherwise the screen will still interpret the image as if it were in the wrong orientation, part of the image may be cut off and the screen will wrap around to the start in rendering.
	pub fn draw(&mut self, img: ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Result<(), crate::errors::ScreenError> {
		if !((img.width() == WIDTH.into() || img.height() == HEIGHT.into()) || (img.width() == HEIGHT.into() || img.height() == WIDTH.into())) {
			// panic!("Canvas size must be 320x480 or 480x320");
			return Err(crate::errors::ScreenError::WrongImageSize);
		}

		let width = img.width();
		let height = img.height();

		// Set the display region
		self.send_command(0, 0, (width - 1) as u16, (height - 1) as u16, ScreenCommand::DisplayBitmap)?;

		let pixels: Vec<_> = img.pixels().collect();
		let width = width as usize;

		for (i, chunk) in pixels.chunks_exact(width * 8).enumerate() {
			let mut bytes: Vec<u8> = Vec::with_capacity(chunk.len() * 2);
			for pixel in chunk {
				let r = (pixel[0] >> 3) as u16;
				let g = (pixel[1] >> 2) as u16;
				let b = (pixel[2] >> 3) as u16;
				let rgb565 = (r << 11) | (g << 5) | b;
				bytes.push((rgb565 & 0xFF) as u8); // LSB
				bytes.push((rgb565 >> 8) as u8); // MSB
			}
			self.port.write(&bytes).map_err(|_| crate::errors::ScreenError::WriteError)?;
		}

		// Write the remaining pixels if any
		let remainder = pixels.chunks_exact(width * 8).remainder();
		if !remainder.is_empty() {
			let mut bytes: Vec<u8> = Vec::with_capacity(remainder.len() * 2);
			for pixel in remainder {
				let r = (pixel[0] >> 3) as u16;
				let g = (pixel[1] >> 2) as u16;
				let b = (pixel[2] >> 3) as u16;
				let rgb565 = (r << 11) | (g << 5) | b;
				bytes.push((rgb565 & 0xFF) as u8); // LSB
				bytes.push((rgb565 >> 8) as u8); // MSB
			}
			self.port.write(&bytes).map_err(|_| crate::errors::ScreenError::WriteError)?;
		}

		Ok(())
	}
}
