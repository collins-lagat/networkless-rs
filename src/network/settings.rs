use anyhow::Result;

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

    pub async fn uuid(&self) -> Result<String> {
        let settings = self.settings.get_settings().await?;
        if let Some(connection) = settings.get("connection") {
            let uuid = connection.get("uuid").unwrap().to_owned();
            let uuid = String::try_from(uuid).unwrap();
            Ok(uuid)
        } else {
            anyhow::bail!("No uuid found")
        }
    }
}
