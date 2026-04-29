use crate::ui::state::{AppContext, EditTarget};
use crate::anima::AnimaWindow;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, CheckButton, Image, Label, Orientation, Scale, ScrolledWindow, Adjustment};
use std::rc::Rc;
use gdk_pixbuf::PixbufAnimation;
use std::cell::RefCell;

pub fn build(ctx: &AppContext, right_scroll: &ScrolledWindow, control_panel: &GtkBox) {
    let control_panel_c = control_panel.clone();
    let right_scroll_c = right_scroll.clone();
    let state = ctx.state.clone();
    let refresh_active_spawns = ctx.refresh_active_spawns.clone();
    let current_edit_target = ctx.current_edit_target.clone();

    let ctx_c = ctx.clone();
    *ctx.update_control_panel.borrow_mut() = Some(Rc::new(move |target| {
        println!("Opening control panel for target...");
        *current_edit_target.borrow_mut() = Some(target.clone());
        for child in control_panel_c.children() { control_panel_c.remove(&child); }

        let s = state.borrow();
        let (name, file_path, config) = match target.clone() {
            EditTarget::Library(id) => {
                let anims = s.db.get_all_animations().unwrap_or_default();
                let a = anims.into_iter().find(|x| x.id == id).expect("Anim not found");
                (a.name, a.file_path, crate::db::InstanceConfig {
                    id: -1, animation_id: id, scale: 1.0, opacity: 1.0, x: 0, y: 0,
                    auto_spawn: false, mirror: false, flip_v: false, roll: 0.0, pitch: 0.0, yaw: 0.0, temperature: 0.0, contrast: 0.0,
                    brightness: 0.0, saturation: 0.0, hue: 0.0
                })
            }
            EditTarget::Instance(id) => {
                let insts = s.db.get_all_instances().unwrap_or_default();
                let i = insts.into_iter().find(|x| x.id == id).expect("Instance not found");
                let anims = s.db.get_all_animations().unwrap_or_default();
                let a = anims.into_iter().find(|x| x.id == i.animation_id).expect("Anim not found");
                (a.name, a.file_path, i)
            }
        };
        drop(s);

        let title = Label::new(None);
        let target_str = match target { EditTarget::Library(_) => "Library Default", EditTarget::Instance(_) => "Active Instance" };
        title.set_markup(&format!("<span size='large' weight='bold'>{} - {}</span>", target_str, name));
        title.set_xalign(0.0);
        control_panel_c.add(&title);

        let preview_img = Image::new();
        preview_img.set_size_request(200, 200);

        let info_label = Label::new(None);
        info_label.set_markup("<span size='small' color='gray'>Preview limited to 1.0x to save space. Actual spawn will be larger.</span>");
        info_label.set_no_show_all(true);

        let preview_path_orig = file_path.clone();

        let update_preview = {
            let preview_img = preview_img.clone();
            let preview_path_orig = preview_path_orig.clone();
            let info_label = info_label.clone();
            move |scale: f64, mirror: bool, flip_v: bool, roll: f64, pitch: f64, yaw: f64, temp: f64, contrast: f64, bright: f64, sat: f64, hue: f64| {
                println!("Rendering preview...");

                if scale > 1.0 {
                    info_label.show();
                } else {
                    info_label.hide();
                }

                let preview_scale = scale.min(1.0);
                let unchanged = (preview_scale - 1.0).abs() < 0.01 && !mirror && !flip_v && roll.abs() < 0.01 && pitch.abs() < 0.01 && yaw.abs() < 0.01 && temp.abs() < 0.01 && contrast.abs() < 0.01 && bright.abs() < 0.01 && sat.abs() < 0.01 && hue.abs() < 0.01;

                if unchanged {
                    match PixbufAnimation::from_file(&preview_path_orig) {
                        Ok(pix) => {
                            preview_img.set_from_animation(&pix);
                            preview_img.show();
                        }
                        Err(e) => eprintln!("Failed to load preview GIF: {}", e),
                    }
                } else {
                    let data = crate::anima_resize::process_gif_in_memory(
                        &preview_path_orig, preview_scale, mirror, flip_v, roll, pitch, yaw, temp, contrast, bright, sat, hue
                    );

                    let loader = gdk_pixbuf::PixbufLoader::with_type("gif").unwrap();
                    if let Err(e) = loader.write(&data) {
                        eprintln!("Failed to write to PixbufLoader: {}", e);
                    }
                    let _ = loader.close();

                    if let Some(anim) = loader.animation() {
                        preview_img.set_from_animation(&anim);
                        preview_img.show();
                    } else {
                        eprintln!("Failed to parse preview GIF from memory");
                    }
                }
            }
        };
        control_panel_c.add(&preview_img);
        control_panel_c.add(&info_label);

        let grid = gtk::Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(20);
        grid.set_margin_top(10);

        let create_slider = |label: &str, min: f64, max: f64, current: f64, step: f64| {
            let l = Label::new(Some(label));
            l.set_xalign(0.0);
            let adj = Adjustment::new(current, min, max, step, step * 10.0, 0.0);
            let sc = Scale::new(Orientation::Horizontal, Some(&adj));
            sc.set_hexpand(true);
            let reset = Button::with_label("↺");
            let adj_c = adj.clone();
            let def_val = if label == "Scale" { 1.0 } else { 0.0 };
            reset.connect_clicked(move |_| adj_c.set_value(def_val));
            (l, sc, adj, reset)
        };

        let (sl_l, sl_s, sl_adj, sl_r) = create_slider("Scale", 0.1, 5.0, config.scale, 0.1);
        grid.attach(&sl_l, 0, 0, 1, 1); grid.attach(&sl_s, 1, 0, 1, 1); grid.attach(&sl_r, 2, 0, 1, 1);

        let (op_l, op_s, op_adj, op_r) = create_slider("Opacity", 0.1, 1.0, config.opacity, 0.05);
        grid.attach(&op_l, 0, 1, 1, 1); grid.attach(&op_s, 1, 1, 1, 1); grid.attach(&op_r, 2, 1, 1, 1);

        let mirror_box = GtkBox::new(Orientation::Horizontal, 5);
        let mirror_check = CheckButton::with_label("Flip H");
        mirror_check.set_active(config.mirror);
        let flip_v_check = CheckButton::with_label("Flip V");
        flip_v_check.set_active(config.flip_v);
        mirror_box.add(&mirror_check);
        mirror_box.add(&flip_v_check);
        grid.attach(&mirror_box, 1, 2, 1, 1);

        let (t_l, t_s, t_adj, t_r) = create_slider("Temp", -100.0, 100.0, config.temperature, 1.0);
        grid.attach(&t_l, 0, 3, 1, 1); grid.attach(&t_s, 1, 3, 1, 1); grid.attach(&t_r, 2, 3, 1, 1);

        let (c_l, c_s, c_adj, c_r) = create_slider("Contrast", -100.0, 100.0, config.contrast, 1.0);
        grid.attach(&c_l, 0, 4, 1, 1); grid.attach(&c_s, 1, 4, 1, 1); grid.attach(&c_r, 2, 4, 1, 1);

        let (b_l, b_s, b_adj, b_r) = create_slider("Brightness", -100.0, 100.0, config.brightness, 1.0);
        grid.attach(&b_l, 0, 5, 1, 1); grid.attach(&b_s, 1, 5, 1, 1); grid.attach(&b_r, 2, 5, 1, 1);

        let (s_l, s_s, s_adj, s_r) = create_slider("Saturation", -100.0, 100.0, config.saturation, 1.0);
        grid.attach(&s_l, 0, 6, 1, 1); grid.attach(&s_s, 1, 6, 1, 1); grid.attach(&s_r, 2, 6, 1, 1);

        let (h_l, h_s, h_adj, h_r) = create_slider("Hue", -180.0, 180.0, config.hue, 1.0);
        grid.attach(&h_l, 0, 7, 1, 1); grid.attach(&h_s, 1, 7, 1, 1); grid.attach(&h_r, 2, 7, 1, 1);

        let (r_l, r_s, r_adj, r_r) = create_slider("Roll (Z)", -180.0, 180.0, config.roll, 5.0);
        grid.attach(&r_l, 0, 8, 1, 1); grid.attach(&r_s, 1, 8, 1, 1); grid.attach(&r_r, 2, 8, 1, 1);

        let (p_l, p_s, p_adj, p_r) = create_slider("Pitch (X)", -90.0, 90.0, config.pitch, 5.0);
        grid.attach(&p_l, 0, 9, 1, 1); grid.attach(&p_s, 1, 9, 1, 1); grid.attach(&p_r, 2, 9, 1, 1);

        let (y_l, y_s, y_adj, y_r) = create_slider("Yaw (Y)", -90.0, 90.0, config.yaw, 5.0);
        grid.attach(&y_l, 0, 10, 1, 1); grid.attach(&y_s, 1, 10, 1, 1); grid.attach(&y_r, 2, 10, 1, 1);

        let auto_spawn_check = CheckButton::with_label("Auto-spawn");
        auto_spawn_check.set_active(config.auto_spawn);
        grid.attach(&auto_spawn_check, 1, 11, 1, 1);

        control_panel_c.add(&grid);

        let live_update_enabled = state.borrow().db.get_live_update_enabled().unwrap_or(true);
        let live_update_delay = state.borrow().db.get_live_update_delay().unwrap_or(300);

        let debounce_id = Rc::new(RefCell::new(None::<gtk::glib::SourceId>));
        let live_update = {
            let up_p = update_preview.clone();
            let sl = sl_adj.clone(); let mir = mirror_check.clone(); let flip_v = flip_v_check.clone();
            let ro = r_adj.clone(); let pi = p_adj.clone(); let ya = y_adj.clone();
            let t = t_adj.clone(); let c = c_adj.clone(); let b = b_adj.clone();
            let s = s_adj.clone(); let h = h_adj.clone();
            let db_id_ref = debounce_id.clone();
            move || {
                if !live_update_enabled { return; }
                if let Some(id) = db_id_ref.borrow_mut().take() { id.remove(); }
                let up_p = up_p.clone();
                let sl_v = sl.value(); let mir_v = mir.is_active(); let fv_v = flip_v.is_active();
                let ro_v = ro.value(); let pi_v = pi.value(); let ya_v = ya.value();
                let t_v = t.value(); let c_v = c.value(); let b_v = b.value();
                let s_v = s.value(); let h_v = h.value();
                let db_id_inner = db_id_ref.clone();
                let id = gtk::glib::timeout_add_local(std::time::Duration::from_millis(live_update_delay), move || {
                    up_p(sl_v, mir_v, fv_v, ro_v, pi_v, ya_v, t_v, c_v, b_v, s_v, h_v);
                    *db_id_inner.borrow_mut() = None;
                    gtk::glib::ControlFlow::Break
                });
                *db_id_ref.borrow_mut() = Some(id);
            }
        };

        sl_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        mirror_check.connect_toggled({let lu = live_update.clone(); move |_| lu()});
        flip_v_check.connect_toggled({let lu = live_update.clone(); move |_| lu()});
        r_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        p_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        y_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        t_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        c_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        b_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        s_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});
        h_adj.connect_value_changed({let lu = live_update.clone(); move |_| lu()});

        update_preview(config.scale, config.mirror, config.flip_v, config.roll, config.pitch, config.yaw, config.temperature, config.contrast, config.brightness, config.saturation, config.hue);

        let action_box = GtkBox::new(Orientation::Horizontal, 10);
        let apply_btn = Button::with_label("Apply Changes");
        let spawn_btn = Button::with_label("Spawn with Settings");
        match target.clone() {
            EditTarget::Library(_) => action_box.add(&spawn_btn),
            EditTarget::Instance(_) => action_box.add(&apply_btn),
        }
        control_panel_c.add(&action_box);

        let state_c = state.clone();
        let refresh_a = refresh_active_spawns.clone();
        let name_c = name.clone();
        let path_c = file_path.clone();

        let effective_target: Rc<RefCell<EditTarget>> = Rc::new(RefCell::new(target.clone()));
        let title_lbl = title.clone();

        let ctx_c2 = ctx_c.clone();
        let on_apply = {
            let effective_target = effective_target.clone();
            move || {
                let mirror = mirror_check.is_active();
                let flip_v = flip_v_check.is_active();
                let roll = r_adj.value();
                let pitch = p_adj.value();
                let yaw = y_adj.value();
                let scale = sl_adj.value();
                let opacity = op_adj.value();
                let temp = t_adj.value();
                let cont = c_adj.value();
                let bright = b_adj.value();
                let sat = s_adj.value();
                let hue = h_adj.value();
                let auto = auto_spawn_check.is_active();

                let mut st = state_c.borrow_mut();

                let db_id = match *effective_target.borrow() {
                    EditTarget::Instance(id) => id,
                    EditTarget::Library(anim_id) => {
                        let id = st.db.insert_instance(anim_id, scale, opacity, 0, 0, auto).unwrap();
                        // Promote to Instance so subsequent applies update, not insert.
                        *effective_target.borrow_mut() = EditTarget::Instance(id);
                        title_lbl.set_markup(&format!(
                            "<span size='large' weight='bold'>Active Instance - {}</span>",
                            name_c
                        ));
                        id
                    }
                };

                let _ = st.db.update_instance_scale(db_id, scale);
                let _ = st.db.update_instance_auto_spawn(db_id, auto);
                let _ = st.db.update_instance_mirror(db_id, mirror);
                let _ = st.db.update_instance_rotation(db_id, flip_v, roll, pitch, yaw);
                let _ = st.db.update_instance_editing(db_id, temp, cont, bright, sat, hue);
                let _ = st.db.update_instance_opacity(db_id, opacity);

                let mut spawn_x = 0i32;
                let mut spawn_y = 0i32;
                if let Some(idx) = st.animas.iter().position(|a| a.instance_db_id == db_id) {
                    let (cx, cy) = st.animas[idx].position();
                    spawn_x = cx;
                    spawn_y = cy;
                    let win = st.animas[idx].window.clone();
                    st.animas.remove(idx);
                    drop(st);
                    win.close();
                    st = state_c.borrow_mut();
                }

                let _ = st.db.update_instance_position(db_id, spawn_x, spawn_y);

                let (counter, g_opacity) = {
                    st.instance_counter += 1;
                    (st.instance_counter, st.global_opacity)
                };
                drop(st);

                let anima = AnimaWindow::new(
                    counter, db_id, name_c.clone(), &path_c,
                    scale, opacity * g_opacity, spawn_x, spawn_y,
                    mirror, flip_v, roll, pitch, yaw, temp, cont, bright, sat, hue
                );
                crate::ui::state::register_anima_window(&ctx_c2, anima);
                if let Some(f) = refresh_a.borrow().as_ref() { f(); }
            }
        };

        let on_apply_rc = Rc::new(on_apply);
        apply_btn.connect_clicked({let oa = on_apply_rc.clone(); move |_| oa()});
        spawn_btn.connect_clicked(move |_| on_apply_rc());

        control_panel_c.show_all();
        right_scroll_c.show_all();
    }));
}
