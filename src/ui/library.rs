use crate::ui::state::{AppContext, EditTarget};
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, EventBox, Label, ListBox, ListBoxRow, Orientation};
use std::rc::Rc;

pub fn build(ctx: &AppContext, library_list: &ListBox) {
    let library_list_c = library_list.clone();
    let state = ctx.state.clone();
    let update_p = ctx.update_control_panel.clone();
    let win_outer = ctx.window.clone();
    
    *ctx.refresh_library.borrow_mut() = Some(Rc::new(move || {
        for child in library_list_c.children() { library_list_c.remove(&child); }
        let anims = state.borrow().db.get_all_animations().unwrap_or_default();
        for anim in anims {
            let row = ListBoxRow::new();
            let anim_id = anim.id;
            let up = update_p.clone();
            
            let ev = EventBox::new();
            row.add(&ev);
            let hbox = GtkBox::new(Orientation::Horizontal, 10);
            ev.add(&hbox);

            let lbl = Label::new(Some(&anim.name));
            lbl.set_xalign(0.0);
            hbox.pack_start(&lbl, true, true, 5);

            ev.connect_button_press_event(move |_, event| {
                if event.button() == 1 {
                    println!("Library row clicked: {}", anim_id);
                    if let Some(f) = up.borrow().as_ref() { f(EditTarget::Library(anim_id)); }
                }
                gtk::glib::Propagation::Proceed
            });

            let del_btn = Button::with_label("Delete");
            let state_del = state.clone();
            let lib_l = library_list_c.clone();
            let r_c = row.clone();
            let win_c = win_outer.clone();
            del_btn.connect_clicked(move |_| {
                let instances = state_del.borrow().db.get_all_instances().unwrap_or_default();
                if instances.iter().any(|i| i.animation_id == anim_id) {
                    let msg = gtk::MessageDialog::new(Some(&win_c), gtk::DialogFlags::MODAL, gtk::MessageType::Error, gtk::ButtonsType::Ok, "Cannot delete animation: it is currently used by one or more instances. Please delete the instances first.");
                    msg.run();
                    msg.close();
                    return;
                }
                
                let msg = gtk::MessageDialog::new(Some(&win_c), gtk::DialogFlags::MODAL, gtk::MessageType::Warning, gtk::ButtonsType::OkCancel, "Are you sure you want to delete this animation?");
                if msg.run() == gtk::ResponseType::Ok {
                    let _ = state_del.borrow().db.delete_animation(anim_id);
                    lib_l.remove(&r_c);
                }
                msg.close();
            });

            hbox.pack_end(&del_btn, false, false, 0);
            library_list_c.add(&row);
        }
        library_list_c.show_all();
    }));
}
