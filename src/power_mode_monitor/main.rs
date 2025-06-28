#![windows_subsystem = "windows"]

use std::{
    collections::HashMap,
    sync::mpsc::{channel, Sender},
    thread::{self},
};

use windows::Win32::System::Power::*;

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use winit::{application::ApplicationHandler, event_loop::EventLoop};

const MENU_ID_EXIT: &str = "exit";
const UNKNOWN_ICO_BLACK: &[u8; 490] = include_bytes!("../../res/unknown_black.ico");
const ONE_ICO_BLACK: &[u8; 276] = include_bytes!("../../res/one_black.ico");
const TWO_ICO_BLACK: &[u8; 578] = include_bytes!("../../res/two_black.ico");
const FOUR_ICO_BLACK: &[u8; 467] = include_bytes!("../../res/four_black.ico");
const UNKNOWN_ICO_WHITE: &[u8; 489] = include_bytes!("../../res/unknown_white.ico");
const ONE_ICO_WHITE: &[u8; 276] = include_bytes!("../../res/one_white.ico");
const TWO_ICO_WHITE: &[u8; 581] = include_bytes!("../../res/two_white.ico");
const FOUR_ICO_WHITE: &[u8; 482] = include_bytes!("../../res/four_white.ico");

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let mut app = Application::new();

    let (tx, rx) = channel::<EFFECTIVE_POWER_MODE>();
    let proxy = event_loop.create_proxy();

    thread::spawn(move || {
        for mode in rx {
            let _ = proxy
                .send_event(UserEvent::PowerModeEvent(mode))
                .expect("Failed to send power mode event");
        }
    });

    let context = Box::new(Context { tx });
    let handle = Box::new(37);

    unsafe {
        let handle_ptr = &mut (Box::into_raw(handle) as *mut core::ffi::c_void);
        let context_ptr = Box::into_raw(context) as *const core::ffi::c_void;

        PowerRegisterForEffectivePowerModeNotifications(
            EFFECTIVE_POWER_MODE_V2,
            Some(effective_power_mode_callback),
            Some(context_ptr),
            handle_ptr,
        )
        .expect("Failed to register for power mode notifications");
    }

    if let Err(err) = event_loop.run_app(&mut app) {
        println!("Error: {:?}", err);
    }
}

unsafe extern "system" fn effective_power_mode_callback(
    mode: EFFECTIVE_POWER_MODE,
    context: *const ::core::ffi::c_void,
) {
    let context = &*(context as *mut Context);
    handle_power_mode_change(mode, context);
}

fn handle_power_mode_change(mode: EFFECTIVE_POWER_MODE, context: &Context) {
    context
        .tx
        .send(mode)
        .expect("Failed to send power mode change");
}

struct Context {
    tx: Sender<EFFECTIVE_POWER_MODE>,
}

#[derive(Debug)]
enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
    PowerModeEvent(EFFECTIVE_POWER_MODE),
}

struct Application {
    tray_icon: Option<TrayIcon>,
    icons_black: HashMap<i32, tray_icon::Icon>,
    icons_white: HashMap<i32, tray_icon::Icon>,
}

impl Application {
    fn new() -> Application {
        Application {
            tray_icon: None,
            icons_black: Self::load_icons_black(),
            icons_white: Self::load_icons_white(),
        }
    }

    fn new_tray_icon(&self) -> TrayIcon {
        let icon = self
            .icons_white
            .get(&0)
            .expect("Failed to load icon")
            .clone();

        TrayIconBuilder::new()
            .with_menu(Box::new(Self::new_tray_menu()))
            .with_tooltip("Power mode: Unknown")
            .with_icon(icon)
            .with_title("x")
            .build()
            .unwrap()
    }

    fn new_tray_menu() -> Menu {
        let menu = Menu::new();
        let item1 = MenuItem::with_id(MENU_ID_EXIT, "Exit", true, None);
        if let Err(err) = menu.append(&item1) {
            println!("{err:?}");
        }
        menu
    }

    fn set_power_mode(&self, power_mode: EFFECTIVE_POWER_MODE) {
        println!("Setting power mode: {:?}", power_mode);
        if let Some(tray_icon) = &self.tray_icon {
            let _ = tray_icon.set_tooltip(Some(format!("Power mode: {}", power_mode.0)));
            if let Some(icon) = self.get_icon(power_mode) {
                let _ = tray_icon.set_icon(Some(icon.clone()));
            }
        }
    }

    fn get_icon(&self, power_mode: EFFECTIVE_POWER_MODE) -> Option<tray_icon::Icon> {
        dark_light::detect()
            .ok()
            .map(|mode| {
                let icons = match mode {
                    dark_light::Mode::Dark => &self.icons_white,
                    dark_light::Mode::Light | dark_light::Mode::Unspecified => &self.icons_black,
                };
                icons.get(&power_mode.0).cloned()
            })
            .flatten()
    }

    fn load_icons_black() -> HashMap<i32, tray_icon::Icon> {
        let mut icons = HashMap::new();
        icons.insert(0, load_icon(UNKNOWN_ICO_BLACK));
        icons.insert(1, load_icon(ONE_ICO_BLACK));
        icons.insert(2, load_icon(TWO_ICO_BLACK));
        icons.insert(4, load_icon(FOUR_ICO_BLACK));
        icons
    }

    fn load_icons_white() -> HashMap<i32, tray_icon::Icon> {
        let mut icons = HashMap::new();
        icons.insert(0, load_icon(UNKNOWN_ICO_WHITE));
        icons.insert(1, load_icon(ONE_ICO_WHITE));
        icons.insert(2, load_icon(TWO_ICO_WHITE));
        icons.insert(4, load_icon(FOUR_ICO_WHITE));
        icons
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        if winit::event::StartCause::Init == cause {
            self.tray_icon = Some(self.new_tray_icon());
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        // println!("Received user event: {:?}", event);
        match event {
            UserEvent::TrayIconEvent(_tray_icon_event) => {}
            UserEvent::MenuEvent(menu_event) => {
                if menu_event.id == MENU_ID_EXIT {
                    println!("Exit menu item clicked, exiting...");
                    std::process::exit(0);
                }
            }
            UserEvent::PowerModeEvent(power_mode) => {
                self.set_power_mode(power_mode);
            }
        }
    }
}

fn load_icon(icon: &[u8]) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(icon)
            .expect("Failed to load icon from memory")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
