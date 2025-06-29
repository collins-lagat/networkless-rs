use std::sync::LazyLock;

use ksni::{Icon, Tray};

use crate::{APP_ID, app::App, trays::get_icon_from_image_bytes};

pub struct AirplaneModeTray {
    app: Option<App>,
}

impl AirplaneModeTray {
    pub fn new() -> Self {
        Self { app: None }
    }

    pub fn set_app(&mut self, app: App) {
        self.app = Some(app);
    }
}

impl Tray for AirplaneModeTray {
    fn id(&self) -> String {
        format!("{}.AirplaneMode", APP_ID)
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut icon = Vec::with_capacity(1);

        static AIRPLANE_MODE_ICON: LazyLock<Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/airplane_mode.png"))
        });

        icon.push(AIRPLANE_MODE_ICON.clone());

        icon
    }
}
