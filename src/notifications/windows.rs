use crate::{Notifier, NotifierError};
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager, ToastNotifier},
};

pub struct WindowsNotifier {
    notifier: ToastNotifier,
}

impl WindowsNotifier {
    pub fn new(app_id: &str) -> Self {
        let app_id = HSTRING::from(app_id);
        WindowsNotifier {
            notifier: ToastNotificationManager::CreateToastNotifierWithId(&app_id)
                .expect("Could not create Notifier with CreateToastNotifierWithId"),
        }
    }
}

impl From<windows::core::Error> for NotifierError {
    fn from(error: windows::core::Error) -> Self {
        NotifierError {
            message: error.to_string().into(),
        }
    }
}

impl Notifier for WindowsNotifier {
    fn send_notification(&self, message: String) -> Result<(), NotifierError> {
        let toast_xml = XmlDocument::new()?;

        toast_xml.LoadXml(&HSTRING::from(format!(
            "<toast duration='short'>
                <visual>
                    <binding template='ToastGeneric'>
                        <text id='1'>{message}</text>
                    </binding>
                </visual>
            </toast>"
        )))?;

        let notification = ToastNotification::CreateToastNotification(&toast_xml)?;
        self.notifier.Show(&notification)?;
        std::thread::sleep(std::time::Duration::from_millis(10)); // Sleep seems to be required

        Ok(())
    }
}
