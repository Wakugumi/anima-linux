pub mod state;
pub mod toolbar;
pub mod library;
pub mod instances;
pub mod control_panel;
pub mod settings;

use crate::anima::AnimaWindow;
use crate::db::Db;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, Label, ListBox, Notebook, Orientation, Paned, Scale, ScrolledWindow, Separator, Adjustment};
use gdk_pixbuf::Pixbuf;
use std::cell::RefCell;
use std::rc::Rc;
use state::{AppContext, AppState};

static APP_ICON_BYTES: &[u8] = include_bytes!("../../icon.png");

fn load_app_icon() -> Option<Pixbuf> {
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(APP_ICON_BYTES).ok()?;
    loader.close().ok()?;
    loader.pixbuf()
}

pub fn build_ui(app: &Application) {
    println!("Building UI...");
    let db = Db::new().expect("Failed to init DB");
    
    let state = Rc::new(RefCell::new(AppState {
        db,
        animas: Vec::new(),
        global_opacity: 1.0,
        instance_counter: 0,
    }));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Anima Advanced Management")
        .default_width(1000)
        .default_height(800)
        .build();

    if let Some(icon) = load_app_icon() {
        window.set_icon(Some(&icon));
    }

    let ctx = AppContext {
        window: window.clone(),
        state: state.clone(),
        refresh_library: Rc::new(RefCell::new(None)),
        refresh_active_spawns: Rc::new(RefCell::new(None)),
        update_control_panel: Rc::new(RefCell::new(None)),
        current_edit_target: Rc::new(RefCell::new(None)),
    };

    let main_vbox = GtkBox::new(Orientation::Vertical, 0);
    
    let toolbar = toolbar::build(&ctx);
    main_vbox.pack_start(&toolbar, false, false, 0);
    main_vbox.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_position(350);

    let left_vbox = GtkBox::new(Orientation::Vertical, 5);
    left_vbox.set_margin(5);

    let notebook = Notebook::new();
    
    let library_list = ListBox::new();
    library_list.set_activate_on_single_click(true);
    let library_scroll = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    library_scroll.add(&library_list);
    library::build(&ctx, &library_list);
    
    let active_spawns_list = ListBox::new();
    active_spawns_list.set_activate_on_single_click(true);
    let spawns_scroll = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    spawns_scroll.add(&active_spawns_list);

    notebook.append_page(&library_scroll, Some(&Label::new(Some("Library"))));
    notebook.append_page(&spawns_scroll, Some(&Label::new(Some("Instances"))));

    let settings_scroll = settings::build(&ctx);
    notebook.append_page(&settings_scroll, Some(&Label::new(Some("Settings"))));

    left_vbox.pack_start(&notebook, true, true, 0);
    
    let opacity_box = GtkBox::new(Orientation::Vertical, 2);
    opacity_box.set_margin(5);
    opacity_box.add(&Label::new(Some("Global Temp Opacity")));
    let opacity_adj = Adjustment::new(1.0, 0.1, 1.0, 0.05, 0.0, 0.0);
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_box.add(&opacity_scale);
    left_vbox.pack_start(&opacity_box, false, false, 0);

    paned.pack1(&left_vbox, false, false);

    let right_scroll = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    let control_panel = GtkBox::new(Orientation::Vertical, 15);
    control_panel.set_margin(20);
    right_scroll.add(&control_panel);
    paned.pack2(&right_scroll, true, false);

    instances::build(&ctx, &active_spawns_list, &control_panel);
    control_panel::build(&ctx, &right_scroll, &control_panel);

    main_vbox.pack_start(&paned, true, true, 0);
    window.add(&main_vbox);

    let state_for_opacity = state.clone();
    opacity_scale.connect_value_changed(move |scale| {
        let val = scale.value();
        let mut s = state_for_opacity.borrow_mut();
        s.global_opacity = val;
        let instances = s.db.get_all_instances().unwrap_or_default();
        for anima in s.animas.iter() {
            if let Some(inst) = instances.iter().find(|i| i.id == anima.instance_db_id) {
                anima.window.set_opacity(inst.opacity * val);
            }
        }
    });

    // Initial Load & Auto-spawn
    if let Some(f) = ctx.refresh_library.borrow().as_ref() { f(); }
    {
        let mut s = state.borrow_mut();
        let max = s.db.get_max_spawns().unwrap_or(10) as usize;
        let instances = s.db.get_all_instances().unwrap_or_default();
        let anims = s.db.get_all_animations().unwrap_or_default();
        
        let mut to_spawn = Vec::new();
        let mut count = 0;
        for inst in instances.iter().filter(|a| a.auto_spawn) {
            if let Some(anim) = anims.iter().find(|a| a.id == inst.animation_id) {
                if count < max {
                    s.instance_counter += 1;
                    to_spawn.push((
                        s.instance_counter, inst.id, anim.name.clone(), anim.file_path.clone(),
                        inst.scale, inst.opacity * s.global_opacity, inst.x, inst.y,
                        inst.mirror, inst.flip_v, inst.roll, inst.pitch, inst.yaw,
                        inst.temperature, inst.contrast, inst.brightness, inst.saturation, inst.hue
                    ));
                    count += 1;
                }
            }
        }
        drop(s);

        for args in to_spawn {
            let anima = AnimaWindow::new(
                args.0, args.1, args.2, &args.3, args.4, args.5, args.6, args.7,
                args.8, args.9, args.10, args.11, args.12, args.13, args.14, args.15, args.16, args.17
            );
            let ctx_clone = ctx.clone();
            crate::ui::state::register_anima_window(&ctx_clone, anima);
        }
    }
    if let Some(f) = ctx.refresh_active_spawns.borrow().as_ref() { f(); }

    // Persistent Position Timer
    let state_t = state.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let s = state_t.borrow();
        for anima in &s.animas {
            let (x, y) = anima.position();
            let _ = s.db.update_instance_position(anima.instance_db_id, x, y);
        }
        gtk::glib::ControlFlow::Continue
    });

    window.show_all();
}
