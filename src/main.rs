use dbus::blocking::Connection;
use std::{env, fs, process};
use std::time::Duration;
use remember_bday::{vcard, Config};
use remember_bday::notifications::DbusNotifier;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let vcards = vcard::parse_vcards(fs::read_to_string(config.file_path).unwrap_or_else(|err| {
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
    let conn = Connection::new_session().expect("Cannot open DBus-Connection");
    let proxy = conn.with_proxy(
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        Duration::from_millis(5000),
    );
    let notifier = DbusNotifier::new(&proxy);

    remember_bday::send_bday_notifications(&notifier, vcards).unwrap_or_else(|err| {
        eprintln!("Problem sending notifications: {}", err);
        process::exit(1);
    })

}
