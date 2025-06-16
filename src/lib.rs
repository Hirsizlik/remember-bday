pub mod notifications;
pub mod vcard;

use chrono::prelude::*;
use notifications::{Notifier, NotifierError};
use vcard::VCard;

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

pub fn send_bday_notifications(
    notifier: &dyn Notifier,
    vcards: Vec<VCard>,
) -> Result<(), NotifierError> {
    // TODO Tests

    let today = chrono::Local::now().date_naive();
    for vcard in vcards {
        if let Some(bday) = vcard.bday {
            if bday.month() == today.month() && bday.day() == today.day() {
                // TODO localization?
                notifier.send_notification(format!("It's {}'s birthday today!", vcard.name))?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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

    struct MockNotifier {
        messages: RefCell<Vec<String>>,
    }

    impl<'a> Notifier for MockNotifier {
        fn send_notification(&self, message: String) -> Result<(), NotifierError> {
            self.messages.borrow_mut().push(message);
            Ok(())
        }
    }

    impl MockNotifier {
        fn new() -> MockNotifier {
            MockNotifier {
                messages: RefCell::new(Vec::new()),
            }
        }
    }

    #[test]
    fn send_bday_notifications_none() {
        let mn = MockNotifier::new();

        // either yesterday or tomorrow, whichever works
        let not_today = chrono::Local::now()
            .date_naive()
            .checked_add_days(chrono::Days::new(1))
            .or_else(|| {
                Some(
                    chrono::Local::now()
                        .date_naive()
                        .checked_sub_days(chrono::Days::new(1))
                        .unwrap(),
                )
            });
        let vcards = vec![
            VCard {
                name: "Test No Birthday".to_string(),
                bday: None,
            },
            VCard {
                name: "Test Birthday not today".to_string(),
                bday: not_today,
            },
        ];
        send_bday_notifications(&mn, vcards).unwrap();
        assert_eq!(Vec::<String>::new(), mn.messages.into_inner());
    }

    #[test]
    fn send_bday_notifications_multiple() {
        let mn = MockNotifier::new();
        let today = chrono::Local::now().date_naive();
        let vcards = vec![
            VCard {
                name: "Test 1".to_string(),
                bday: Some(today),
            },
            VCard {
                name: "Test 2".to_string(),
                bday: Some(today),
            },
        ];
        send_bday_notifications(&mn, vcards).unwrap();
        assert_eq!(
            vec![
                "It's Test 1's birthday today!".to_string(),
                "It's Test 2's birthday today!".to_string()
            ],
            mn.messages.into_inner()
        );
    }
}
