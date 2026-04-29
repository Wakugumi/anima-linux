mod anima;
mod ui;
mod db;
mod anima_resize;
mod env_detect;

use gio::prelude::*;
use gtk::Application;

fn main() {
    let app = Application::builder()
        .application_id("com.github.zylquinal.anima-linux")
        .build();

    app.connect_activate(|app| {
        ui::build_ui(app);
    });

    app.run();
}
