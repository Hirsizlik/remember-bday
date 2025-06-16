#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod linux_gen;

#[cfg(target_os = "windows")]
pub mod windows;

use std::error;
use std::fmt;

pub trait Notifier {
    fn send_notification(&self, message: String) -> Result<(), NotifierError>;
}

#[derive(Debug)]
pub struct NotifierError {
    message: Box<str>,
}

impl error::Error for NotifierError {}

impl fmt::Display for NotifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Error while sending a notification: {}", self.message)
    }
}
