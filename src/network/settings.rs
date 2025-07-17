use anyhow::Result;
use zbus::zvariant::ObjectPath;

use crate::interfaces::settings::connection::ConnectionProxy;

pub struct ConnectionSetting {
    settings: ConnectionProxy<'static>,
}

impl ConnectionSetting {
    pub fn new(settings: ConnectionProxy<'static>) -> Self {
        Self { settings }
    }

    pub async fn id(&self) -> Result<String> {
        let settings = self.settings.get_settings().await?;
        if let Some(connection) = settings.get("connection") {
            let id = connection.get("id").unwrap().to_owned();
            let id = String::try_from(id).unwrap();
            Ok(id)
        } else {
            anyhow::bail!("No id found")
        }
    }

    pub fn path(&self) -> ObjectPath<'static> {
        self.settings.inner().path().clone()
    }
}
