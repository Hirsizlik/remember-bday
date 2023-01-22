pub mod gen;

use dbus::arg::{PropMap, Variant};
use dbus::blocking::{Connection, Proxy};
use gen::OrgFreedesktopPortalNotification;
use super::{Notifier, NotifierError};


pub struct DbusNotifier<'a> {
    proxy: &'a Proxy<'a, &'a Connection>
}

impl<'a> DbusNotifier<'a> {
    pub fn new(proxy: &'a Proxy<'a, &'a Connection>) -> Self {
        DbusNotifier {proxy}
    }
}

impl<'a> Notifier for DbusNotifier<'a> {
    fn send_notification(&self, message: String) -> Result<(), NotifierError> {
        let mut map = PropMap::new();
        map.insert(
            String::from("title"),
            Variant(Box::new(String::from("Remember B-Day"))),
        );
        map.insert(String::from("body"), Variant(Box::new(message)));
        // TODO icon (cake)
        self.proxy.add_notification("my-id", map)?;
        Ok(())
    }
}

impl From<dbus::Error> for NotifierError {
    fn from(error: dbus::Error) -> Self {
        NotifierError { message: error.message().unwrap_or("unknown").into() }
    }
}
