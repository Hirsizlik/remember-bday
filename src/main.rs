use remember_bday::notifications::Notifier;
use remember_bday::{vcard, Config};
use std::{env, fs, process};

#[cfg(target_os = "linux")]
fn create_notifier(_: &Config) -> impl Notifier {
    let conn = dbus::blocking::Connection::new_session().expect("Cannot open DBus-Connection");
    remember_bday::notifications::linux::DbusNotifier::new(conn)
}

#[cfg(target_os = "windows")]
fn create_notifier(config: &Config) -> impl Notifier {
    remember_bday::notifications::windows::WindowsNotifier::new(&config.windows_app_id)
}

fn main() {
    let config = Config::build(env::args(), env::vars()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let vcards = vcard::parse_vcards(fs::read_to_string(&config.file_path).unwrap_or_else(|err| {
        eprintln!("Problem reading file: {}", err);
        process::exit(1);
    }));

    let vcards = match vcards {
        Ok(vcards) => vcards,
        Err(message) => {
            eprintln!("{}", message);
            return;
        }
    };
    let notifier = create_notifier(&config);

    remember_bday::send_bday_notifications(&notifier, vcards).unwrap_or_else(|err| {
        eprintln!("Problem sending notifications: {}", err);
        process::exit(1);
    })
}
