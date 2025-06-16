use std::time::Duration;

use crate::notifications::linux_gen::OrgFreedesktopPortalNotification;
use crate::{Notifier, NotifierError};
use dbus::arg::{PropMap, Variant};
use dbus::blocking::Connection;

pub struct DbusNotifier {
    conn: Connection,
}

impl<'a> DbusNotifier {
    pub fn new(conn: Connection) -> Self {
        DbusNotifier { conn }
    }
}

impl<'a> Notifier for DbusNotifier {
    fn send_notification(&self, message: String) -> Result<(), NotifierError> {
        let proxy = self.conn.with_proxy(
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            Duration::from_millis(5000),
        );

        let mut map = PropMap::new();
        map.insert(
            String::from("title"),
            Variant(Box::new(String::from("Remember B-Day"))),
        );
        map.insert(String::from("body"), Variant(Box::new(message)));
        map.insert(
            String::from("priority"),
            Variant(Box::new(String::from("low"))),
        );
        // TODO icon (cake)
        proxy.add_notification("my-id", map)?;
        Ok(())
    }
}

impl From<dbus::Error> for NotifierError {
    fn from(error: dbus::Error) -> Self {
        NotifierError {
            message: error.message().unwrap_or("unknown").into(),
        }
    }
}
