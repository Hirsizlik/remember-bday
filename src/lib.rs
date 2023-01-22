pub mod notifications;
pub mod vcard;

use std::fmt;
use std::error;
use vcard::VCard;
use chrono::prelude::*;

pub struct Config {
    pub file_path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) if arg.ends_with(".vcf") => arg,
            _ => return Err("Didn't get path to a vcf file"),
        };

        Ok(Config { file_path })
    }
}

pub trait Notifier {
    fn send_notification(&self, message: String) -> Result<(), NotifierError>;
}

#[derive(Debug)]
pub struct NotifierError {
    message: Box<str>
}

impl error::Error for NotifierError {}

impl fmt::Display for NotifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Error while sending a notification: {}", self.message)
    }
}

pub fn send_bday_notifications(notifier: &dyn Notifier, vcards: Vec<VCard>) -> Result<(), NotifierError> {
    // TODO refactor to trait for connection/proxy + Tests

    let today = chrono::Local::now().date_naive();
    for vcard in vcards {
        if let Some(bday) = vcard.bday {
            if bday.month() == today.month() && bday.day() == today.day() {
                // TODO localization?
                notifier.send_notification(
                    format!("It's {}'s birthday today!", vcard.name),
                )?;
            }
        }
    }

    Ok(())
}

/*
fn send_notification(proxy: &Proxy<&Connection>, message: String) -> Result<(), dbus::Error> {
    let mut map = PropMap::new();
    map.insert(
        String::from("title"),
        Variant(Box::new(String::from("Remember B-Day"))),
    );
    map.insert(String::from("body"), Variant(Box::new(message)));
    // TODO icon (cake)
    proxy.add_notification("my-id", map)?;
    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_config_success() {
        let test_args = vec!["ignored", "/a/path/to/a.vcf", "also ignored"];
        let config = Config::build(test_args.iter().map(|s| String::from(*s)));
        assert_eq!("/a/path/to/a.vcf", config.unwrap().file_path);
    }

    #[test]
    fn build_config_failure_noarg() {
        let test_args = vec!["ignored"];
        let config = Config::build(test_args.iter().map(|s| String::from(*s)));
        assert!(config.is_err());
    }

    #[test]
    fn build_config_failure_wrong_type() {
        let test_args = vec!["ignored", "/a/path/to/a.txt", "also ignored"];
        let config = Config::build(test_args.iter().map(|s| String::from(*s)));
        assert!(config.is_err());
    }
}
