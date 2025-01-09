pub mod errors;
pub mod screen;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_screen() {
		let port = screen::Screen::find_port().expect("No port found");
		let mut screen = screen::Screen::new(port).expect("Failed to open port");

		screen
			.orientation(screen::Orientation::Portrait)
			.expect("Failed to set orientation");
		screen.clear().expect("Failed to clear screen");

		let img = image::ImageReader::open("meme.png")
			.unwrap()
			.decode()
			.unwrap();
		screen.draw(img.into()).expect("Failed to draw image");

		screen.brightness(10).expect("Failed to set brightness");
		screen.screen_off().expect("Failed to turn screen off");
		std::thread::sleep(core::time::Duration::from_secs(1));
		screen.screen_on().expect("Failed to turn screen on");
		screen.brightness(255).expect("Failed to set brightness");
		screen.to_black().expect("Failed to turn screen black");
	}
}
