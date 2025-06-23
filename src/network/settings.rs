use crate::interfaces::settings::connection::ConnectionProxy;

pub struct ConnectionSetting {
    settings: ConnectionProxy<'static>,
}

impl ConnectionSetting {
    pub fn new(settings: ConnectionProxy<'static>) -> Self {
        Self { settings }
    }
}
