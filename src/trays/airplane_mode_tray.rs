use std::sync::LazyLock;

use ksni::{Icon, Tray, menu::CheckmarkItem};

use crate::{
    APP_ID,
    app::{Action, App},
    trays::get_icon_from_image_bytes,
};

pub struct AirplaneModeTray {
    app: App,
}

impl AirplaneModeTray {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}

impl Tray for AirplaneModeTray {
    fn id(&self) -> String {
        format!("{}.AirplaneMode", APP_ID)
    }

    fn title(&self) -> String {
        "Airplane Mode".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut icon = Vec::with_capacity(1);

        static AIRPLANE_MODE_ICON: LazyLock<Icon> = LazyLock::new(|| {
            get_icon_from_image_bytes(include_bytes!("../../assets/airplane_mode.png"))
        });

        icon.push(AIRPLANE_MODE_ICON.clone());

        icon
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            CheckmarkItem {
                label: "On".into(),
                checked: true,
                activate: Box::new(|this: &mut Self| {
                    this.app
                        .send_action_blocking(Action::ToggleAirplaneMode(false));
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
