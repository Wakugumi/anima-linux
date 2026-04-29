use crate::ui::state::{AppContext, EditTarget};
use crate::anima::AnimaWindow;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, EventBox, Label, ListBox, ListBoxRow, Orientation};
use std::rc::Rc;

pub fn build(ctx: &AppContext, active_spawns_list: &ListBox, control_panel: &GtkBox) {
    let active_spawns_list_c = active_spawns_list.clone();
    let state = ctx.state.clone();
    let update_p = ctx.update_control_panel.clone();
    let refresh_inner = ctx.refresh_active_spawns.clone();
    let current_edit_target_list = ctx.current_edit_target.clone();
    let cp_list = control_panel.clone();
    let ctx_outer = ctx.clone();
    
    *ctx.refresh_active_spawns.borrow_mut() = Some(Rc::new(move || {
        for child in active_spawns_list_c.children() { active_spawns_list_c.remove(&child); }
        let s = state.borrow();
        let instances = s.db.get_all_instances().unwrap_or_default();
        let anims = s.db.get_all_animations().unwrap_or_default();
        for inst in instances {
            let anim = match anims.iter().find(|a| a.id == inst.animation_id) { Some(a) => a, None => continue };
            let running_opt = s.animas.iter().find(|x| x.instance_db_id == inst.id);
            let is_running = running_opt.is_some();
            let running_id = running_opt.map(|x| x.id).unwrap_or(0);

            let row = ListBoxRow::new();
            let db_id = inst.id;
            let up = update_p.clone();
            
            let ev = EventBox::new();
            row.add(&ev);
            let hbox = GtkBox::new(Orientation::Horizontal, 10);
            ev.add(&hbox);

            let lbl = Label::new(Some(&format!("{} (ID:{}){}", anim.name, inst.id, if is_running { " [Running]" } else { "" })));
            lbl.set_xalign(0.0);
            hbox.pack_start(&lbl, true, true, 5);

            ev.connect_button_press_event(move |_, event| {
                if event.button() == 1 {
                    println!("Active row clicked: {}", db_id);
                    if let Some(f) = up.borrow().as_ref() { f(EditTarget::Instance(db_id)); }
                }
                gtk::glib::Propagation::Proceed
            });
            
            let btn_box = GtkBox::new(Orientation::Horizontal, 5);

            if is_running {
                let loc_btn = Button::with_label("Locate");
                let state_loc = state.clone();
                loc_btn.connect_clicked(move |_| {
                    if let Some(a) = state_loc.borrow().animas.iter().find(|x| x.id == running_id) { a.locate(); }
                });
                btn_box.pack_start(&loc_btn, false, false, 0);

                let des_btn = Button::with_label("Despawn");
                let state_des = state.clone();
                let ref_a = refresh_inner.clone();
                des_btn.connect_clicked(move |_| {
                    let mut st = state_des.borrow_mut();
                    if let Some(idx) = st.animas.iter().position(|x| x.instance_db_id == db_id) {
                        let win = st.animas[idx].window.clone();
                        st.animas.remove(idx);
                        drop(st);
                        win.close();
                    } else {
                        drop(st);
                    }
                    if let Some(f) = ref_a.borrow().as_ref() { f(); }
                });
                btn_box.pack_start(&des_btn, false, false, 0);
            } else {
                let spawn_btn = Button::with_label("Spawn");
                let state_spawn = state.clone();
                let ref_a = refresh_inner.clone();
                let ctx_clone = ctx_outer.clone();
                spawn_btn.connect_clicked(move |_| {
                    let (inst_data, anim_path, anim_name) = {
                        let st = state_spawn.borrow();
                        let insts = st.db.get_all_instances().unwrap_or_default();
                        let anims = st.db.get_all_animations().unwrap_or_default();
                        let inst = match insts.into_iter().find(|i| i.id == db_id) {
                            Some(i) => i,
                            None => return, // instance was deleted
                        };
                        let anim = match anims.into_iter().find(|a| a.id == inst.animation_id) {
                            Some(a) => a,
                            None => return,
                        };
                        (inst, anim.file_path, anim.name)
                    };
                    let (counter, g_opacity) = {
                        let mut st = state_spawn.borrow_mut();
                        st.instance_counter += 1;
                        (st.instance_counter, st.global_opacity)
                    };
                    let anima = AnimaWindow::new(
                        counter, db_id, anim_name, &anim_path,
                        inst_data.scale, inst_data.opacity * g_opacity,
                        inst_data.x, inst_data.y,
                        inst_data.mirror, inst_data.flip_v,
                        inst_data.roll, inst_data.pitch, inst_data.yaw,
                        inst_data.temperature, inst_data.contrast,
                        inst_data.brightness, inst_data.saturation, inst_data.hue
                    );
                    let ctx_clone = ctx_clone.clone();
                    crate::ui::state::register_anima_window(&ctx_clone, anima);
                    if let Some(f) = ref_a.borrow().as_ref() { f(); }
                });
                btn_box.pack_start(&spawn_btn, false, false, 0);
            }


            let del_btn = Button::with_label("Delete");
            let state_del = state.clone();
            let ref_a2 = refresh_inner.clone();
            let cur_target = current_edit_target_list.clone();
            let cp = cp_list.clone();
            del_btn.connect_clicked(move |_| {
                let mut st = state_del.borrow_mut();
                if let Some(idx) = st.animas.iter().position(|x| x.instance_db_id == db_id) {
                    let win = st.animas[idx].window.clone();
                    st.animas.remove(idx);
                    let _ = st.db.delete_instance(db_id);
                    drop(st);
                    win.close();
                } else {
                    let _ = st.db.delete_instance(db_id);
                    drop(st);
                }
                
                if let Some(EditTarget::Instance(id)) = *cur_target.borrow() {
                    if id == db_id {
                        for child in cp.children() { cp.remove(&child); }
                    }
                }
                if let Some(f) = ref_a2.borrow().as_ref() { f(); }
            });
            btn_box.pack_start(&del_btn, false, false, 0);
            
            hbox.pack_end(&btn_box, false, false, 0);
            active_spawns_list_c.add(&row);
        }
        active_spawns_list_c.show_all();
    }));
}
