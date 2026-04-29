use crate::ui::state::AppContext;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, Entry, FileChooserAction, FileChooserDialog, Label, MessageDialog, Orientation, ResponseType, SpinButton, Adjustment};

pub fn build(ctx: &AppContext) -> GtkBox {
    let toolbar = GtkBox::new(Orientation::Horizontal, 10);
    toolbar.set_margin(10);
    
    let import_btn = Button::with_label("Import Anima");
    
    let max_spawns = ctx.state.borrow().db.get_max_spawns().unwrap_or(10);
    let max_spawns_box = GtkBox::new(Orientation::Horizontal, 5);
    max_spawns_box.add(&Label::new(Some("Max Spawns:")));
    let max_spawns_adj = Adjustment::new(max_spawns as f64, 1.0, 100.0, 1.0, 10.0, 0.0);
    let max_spawns_spin = SpinButton::new(Some(&max_spawns_adj), 1.0, 0);
    max_spawns_box.add(&max_spawns_spin);
    
    toolbar.pack_start(&import_btn, false, false, 0);
    toolbar.pack_end(&max_spawns_box, false, false, 0);
    
    let state_for_max = ctx.state.clone();
    max_spawns_spin.connect_value_changed(move |spin| {
        let val = spin.value() as i32;
        let _ = state_for_max.borrow().db.set_max_spawns(val);
    });

    let win_c = ctx.window.clone();
    let state_i = ctx.state.clone();
    let ref_l = ctx.refresh_library.clone();
    import_btn.connect_clicked(move |_| {
        let fc = FileChooserDialog::new(Some("Import Anima"), Some(&win_c), FileChooserAction::Open);
        fc.add_buttons(&[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)]);
        if fc.run() == ResponseType::Accept {
            if let Some(path) = fc.filename() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.len() > 10 * 1024 * 1024 {
                        let warn_diag = MessageDialog::new(Some(&win_c), gtk::DialogFlags::MODAL, gtk::MessageType::Warning, gtk::ButtonsType::YesNo, "This file is very large (>10MB). Processing large/long GIFs can be very slow and use a lot of memory. Are you sure you want to import it?");
                        let res = warn_diag.run();
                        warn_diag.close();
                        if res != ResponseType::Yes {
                            fc.close();
                            return;
                        }
                    }
                }
                
                let nd = MessageDialog::new(Some(&win_c), gtk::DialogFlags::MODAL, gtk::MessageType::Question, gtk::ButtonsType::OkCancel, "Name:");
                let entry = Entry::new();
                entry.set_text(&path.file_stem().unwrap().to_string_lossy());
                nd.content_area().pack_start(&entry, true, true, 0);
                nd.show_all();
                if nd.run() == ResponseType::Ok {
                    let dest = crate::db::Db::app_dir().join(path.file_name().unwrap());
                    if std::fs::copy(&path, &dest).is_ok() {
                        let _ = state_i.borrow().db.insert_animation(&entry.text(), dest.to_str().unwrap());
                        if let Some(f) = ref_l.borrow().as_ref() { f(); }
                    }
                }
                nd.close();
            }
        }
        fc.close();
    });

    toolbar
}
