#[cfg(feature = "scripting")]
use std::path::Path;

use dynamo_common::prelude::*;
use dynamo_core::prelude::*;
use dynamo_gui::hotkeys::{
    ANNOTATION_HOTKEYS, CYCLES_HOTKEYS, FILE_HOTKEYS, Hotkey, IMAGE_HOTKEYS, INCOLORING_HOTKEYS,
    OUTCOLORING_HOTKEYS, PALETTE_HOTKEYS, SELECTION_HOTKEYS,
};
use dynamo_gui::interface::{Interface, MainInterface};
use dynamo_profiles::Mandelbrot;
use egui::Ui;
use egui_dock::{NodeIndex, SurfaceIndex};
#[cfg(feature = "scripting")]
use script_loader::error::ScriptError;

#[cfg(feature = "scripting")]
use crate::script_editor::*;
use crate::sidebar;

#[derive(Clone, Copy, Default, Debug)]
pub enum MenuState
{
    #[default]
    Closed,
    Open,
}
impl MenuState
{
    pub const fn close(&mut self)
    {
        *self = Self::Closed;
    }
    pub const fn open(&mut self)
    {
        *self = Self::Open;
    }
    #[must_use]
    pub const fn is_open(&self) -> bool
    {
        matches!(self, Self::Open)
    }
    #[must_use]
    pub const fn is_closed(&self) -> bool
    {
        matches!(self, Self::Closed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabID
{
    pub surface: SurfaceIndex,
    pub node:    NodeIndex,
}
impl Default for TabID
{
    fn default() -> Self
    {
        Self {
            surface: SurfaceIndex::main(),
            node:    NodeIndex(0),
        }
    }
}

impl From<TabID> for (SurfaceIndex, NodeIndex)
{
    fn from(value: TabID) -> Self
    {
        (value.surface, value.node)
    }
}

pub struct FractalTab
{
    pub interface: Box<dyn Interface>,
    pub id: TabID,
    pub menu_state: MenuState,
    pub sidebar_menu: sidebar::menu::Menu,
    #[cfg(feature = "scripting")]
    pub popup: Option<Popup>,
    #[cfg(feature = "scripting")]
    pub error_report: Option<ErrorReport>,
}

impl FractalTab
{
    #[must_use]
    pub const fn with_id(mut self, tab_id: TabID) -> Self
    {
        self.id = tab_id;
        self
    }

    pub fn update(&mut self, ui: &mut Ui)
    {
        egui::SidePanel::left("Fractal")
            .default_width(220.)
            .show_inside(ui, |ui| {
                self.sidebar(ui);
            });

        if self.should_update_interface() {
            self.interface.update(ui.ctx());
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(self.interface.name());
            self.show_menu(ui);
            self.interface.show(ui);
        });

        #[cfg(feature = "scripting")]
        self.show_popup(ui);
    }

    fn show_menu(&mut self, ui: &mut Ui)
    {
        self.menu_state.close();
        egui::MenuBar::new().ui(ui, |ui| {
            self.file_menu(ui);
            self.image_menu(ui);
            self.selection_menu(ui);
            self.annotations_menu(ui);
            self.coloring_menu(ui);
            #[cfg(feature = "scripting")]
            self.transpiled_scripts_menu(ui);
            self.help_menu(ui);
        });
    }

    fn file_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("File", |ui| {
            self.menu_state.open();
            for hotkey in &FILE_HOTKEYS {
                self.hotkey_button(ui, hotkey);
            }
        });
    }

    fn sidebar(&mut self, ui: &mut Ui)
    {
        use sidebar::menu::Action::ChangeFractal;
        if let Some(action) = self.sidebar_menu.show_and_get_action(ui) {
            match action {
                ChangeFractal(interface) => {
                    self.interface = interface;
                }
            }
        }
    }

    fn coloring_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Coloring", |ui| {
            self.menu_state.open();
            ui.menu_button("Palette", |ui| {
                for hotkey in &PALETTE_HOTKEYS {
                    self.hotkey_button(ui, hotkey);
                }
            });

            ui.menu_button("Incoloring", |ui| {
                for hotkey in &INCOLORING_HOTKEYS {
                    self.hotkey_button(ui, hotkey);
                }
            });
            for hotkey in &OUTCOLORING_HOTKEYS {
                self.hotkey_button(ui, hotkey);
            }
        });
    }

    fn image_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Image", |ui| {
            self.menu_state.open();
            ui.menu_button("Set height", |ui| {
                if ui.button("256").clicked() {
                    self.interface.change_height(256);
                } else if ui.button("512").clicked() {
                    self.interface.change_height(512);
                } else if ui.button("768").clicked() {
                    self.interface.change_height(768);
                } else if ui.button("1024").clicked() {
                    self.interface.change_height(1024);
                } else if ui.button("1536").clicked() {
                    self.interface.change_height(1536);
                } else {
                    return;
                }
                self.interface.consume_click();
                ui.close();
            });

            for hotkey in &IMAGE_HOTKEYS {
                self.hotkey_button(ui, hotkey);
            }
        });
    }

    fn selection_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Selection", |ui| {
            self.menu_state.open();
            for hotkey in &SELECTION_HOTKEYS {
                self.hotkey_button(ui, hotkey);
            }
        });
    }

    fn annotations_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("Annotations", |ui| {
            self.menu_state.open();
            ui.menu_button("Cycles", |ui| {
                for hotkey in &CYCLES_HOTKEYS {
                    self.hotkey_button(ui, hotkey);
                }
            });
            for hotkey in &ANNOTATION_HOTKEYS {
                self.hotkey_button(ui, hotkey);
            }
        });
    }

    #[cfg(feature = "scripting")]
    fn transpiled_scripts_menu(&mut self, ui: &mut Ui)
    {
        ui.menu_button("User Scripts", |ui| {
            if ui.button("New script").clicked() {
                self.popup = Some(Popup::new_script());
                ui.close();
            }
            if ui.button("Edit script...").clicked() {
                self.popup = Some(Popup::load_edit());
                ui.close();
            }
            if ui.button("Load script...").clicked() {
                self.popup = Some(Popup::load());
                ui.close();
            }
        });
    }

    #[cfg(feature = "scripting")]
    fn handle_popup_response(&mut self, response: Response)
    {
        use Response::*;
        match response {
            DoNothing => {}
            Close => {
                self.popup = None;
            }
            Load(path) => match self.load_user_script(path) {
                Ok(()) => {
                    self.popup = None;
                }
                Err(e) => {
                    self.error_report = Some(ErrorReport::new(
                        "Error parsing script".to_owned(),
                        format!("{e:?}"),
                    ));
                }
            },
        }
    }

    #[cfg(feature = "scripting")]
    fn should_update_interface(&self) -> bool
    {
        self.popup.is_none() && self.menu_state.is_closed()
    }

    #[cfg(not(feature = "scripting"))]
    const fn should_update_interface(&self) -> bool
    {
        self.menu_state.is_closed()
    }

    #[allow(clippy::unused_self)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    const fn help_menu(&mut self, _ui: &mut Ui)
    {
        // TODO: create help menu
        // ui.menu_button("Help", |ui| {
        //     if ui.button("About").clicked() {
        //         self.show_about_window()
        //     }
        // });
    }

    #[cfg(feature = "scripting")]
    fn load_user_script<P: AsRef<Path>>(&mut self, script_path: P) -> Result<(), ScriptError>
    {
        use script_loader::Loader;
        let image_height = self.interface.get_image_height();
        let loader = Loader::new(script_path.as_ref(), image_height);
        unsafe {
            let int = loader.run()?;
            self.interface = Box::new(int);
        }
        Ok(())
    }

    fn hotkey_button(&mut self, ui: &mut Ui, hotkey: &Hotkey)
    {
        if let Some(action) = hotkey.menu_action()
            && ui
                .add(
                    egui::Button::new(action.short_description())
                        .shortcut_text(hotkey.shortcut_text().unwrap_or_default()),
                )
                .clicked()
        {
            self.interface.process_action(action);
            self.interface.consume_click();
            ui.close();
        }
    }

    #[cfg(feature = "scripting")]
    fn show_popup(&mut self, ui: &mut Ui)
    {
        if let Some(popup) = self.popup.as_mut() {
            popup.show(ui.ctx());
            let response = popup.pop_response();
            self.handle_popup_response(response);
        }
        if let Some(error_report) = self.error_report.as_mut() {
            error_report.show(ui.ctx());
            if !error_report.visible {
                self.error_report = None;
            }
        }
    }
}

impl Default for FractalTab
{
    fn default() -> Self
    {
        type Profile = Mandelbrot;

        let height = IMAGE_HEIGHT;

        let parent_plane = Profile::default().with_res_y(height).with_max_iter(1024);
        let child_plane = JuliaSet::from(parent_plane.clone());

        let interface = Box::new(MainInterface::new(parent_plane, child_plane, height));
        let sidebar_menu = sidebar::create_menu();

        Self {
            interface,
            sidebar_menu,
            menu_state: MenuState::default(),
            id: TabID::default(),
            #[cfg(feature = "scripting")]
            popup: None,
            #[cfg(feature = "scripting")]
            error_report: None,
        }
    }
}
