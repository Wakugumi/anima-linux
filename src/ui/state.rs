use crate::anima::AnimaWindow;
use crate::db::Db;
use gtk::ApplicationWindow;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct AppState {
    pub db: Db,
    pub animas: Vec<AnimaWindow>,
    pub global_opacity: f64,
    pub instance_counter: usize,
}

#[derive(Clone)]
pub enum EditTarget {
    Library(i32),  // Animation ID
    Instance(i32), // Instance ID
}

impl PartialEq for EditTarget {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EditTarget::Library(a), EditTarget::Library(b)) => a == b,
            (EditTarget::Instance(a), EditTarget::Instance(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub window: ApplicationWindow,
    pub state: Rc<RefCell<AppState>>,
    pub refresh_library: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    pub refresh_active_spawns: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
    pub update_control_panel: Rc<RefCell<Option<Rc<dyn Fn(EditTarget)>>>>,
    pub current_edit_target: Rc<RefCell<Option<EditTarget>>>,
}

pub fn register_anima_window(ctx: &AppContext, anima: AnimaWindow) {
    let w = anima.window.clone();
    let st_c = ctx.state.clone();
    let ref_a_c = ctx.refresh_active_spawns.clone();
    let spawn_id = anima.id;

    w.connect_destroy(move |_| {
        if let Ok(mut s) = st_c.try_borrow_mut() {
            if let Some(idx) = s.animas.iter().position(|x| x.id == spawn_id) {
                s.animas.remove(idx);
                drop(s);
                if let Ok(f_ref) = ref_a_c.try_borrow() {
                    if let Some(f) = f_ref.as_ref() { f(); }
                }
            }
        }
    });

    ctx.state.borrow_mut().animas.push(anima);
}
