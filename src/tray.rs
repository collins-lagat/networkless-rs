use std::sync::LazyLock;

use image::GenericImageView;

use crate::APP_ID;

pub struct Tray {}

impl Tray {
    pub fn new() -> Self {
        Self {}
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        APP_ID.into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        static WIFI_100_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
            let img = image::load_from_memory_with_format(
                include_bytes!("../assets/wifi-100.png"),
                image::ImageFormat::Png,
            )
            .expect("valid image");
            let (width, height) = img.dimensions();
            let mut data = img.into_rgba8().into_vec();
            assert_eq!(data.len() % 4, 0);
            for pixel in data.chunks_exact_mut(4) {
                pixel.rotate_right(1) // rgba to argb
            }
            ksni::Icon {
                width: width as i32,
                height: height as i32,
                data,
            }
        });

        vec![WIFI_100_ICON.clone()]
    }

    fn title(&self) -> String {
        "Networkless".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::SubMenu;
        vec![
            SubMenu {
                label: "wireless".into(),
                submenu: vec![],
                ..Default::default()
            }
            .into(),
        ]
    }
}
