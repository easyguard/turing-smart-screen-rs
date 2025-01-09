use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenError {
	#[error("Error writing data to screen")]
	WriteError,
	#[error("Wrong image size; must be 320x480 or 480x320")]
	WrongImageSize
}