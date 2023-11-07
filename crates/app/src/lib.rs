#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

pub mod fractal_tab;
pub mod macros;
#[cfg(feature = "scripting")]
pub mod script_editor;
use fractal_tab::{FractalTab, TabID};

#[cfg(not(target_arch = "wasm32"))]
pub fn run_app() -> Result<(), eframe::Error>
{
    use dynamo_common::prelude::{WIN_HEIGHT, WIN_WIDTH};

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WIN_WIDTH, WIN_HEIGHT)),
        run_and_return: false,
        ..Default::default()
    };

    eframe::run_native(
        "Dynamo",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<FractalApp>::default()
        }),
    )
}

struct TabViewer<'a>
{
    added_nodes: &'a mut Vec<FractalTab>,
    to_remove: &'a mut Vec<TabID>,
}

impl egui_dock::TabViewer for TabViewer<'_>
{
    type Tab = FractalTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab)
    {
        use dynamo_gui::interface::UiMessage::*;

        tab.update(ui);
        match tab.interface.pop_message() {
            Quit => {
                std::process::exit(0);
            }
            CloseWindow => {
                self.to_remove.push(tab.id);
            }
            NewTab => {
                self.on_add(tab.id.surface, tab.id.node);
            }
            DoNothing => {}
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText
    {
        format!("Tab {}", tab.interface.name()).into()
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex)
    {
        let tab_id = TabID { surface, node };
        let tab = FractalTab::default().with_id(tab_id);
        self.added_nodes.push(tab);
    }
}

pub struct FractalApp
{
    dock_state: DockState<FractalTab>,
    tab_count: usize,
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let tab0 = FractalTab::default();

        let dock_state = DockState::new(vec![tab0]);

        Self {
            dock_state,
            tab_count: 1,
        }
    }
}

impl eframe::App for FractalApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        let mut added_nodes = Vec::new();
        let mut to_remove = Vec::new();
        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tab_bar.fill_tab_bar = true;
                style
            })
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                    to_remove: &mut to_remove,
                },
            );
        for tab in added_nodes {
            self.dock_state.set_focused_node_and_surface(tab.id.into());
            self.dock_state.push_to_focused_leaf(tab);
            self.tab_count += 1;
        }
        for tab_id in to_remove {
            self.tab_count -= 1;
            if self.tab_count == 0 {
                std::process::exit(0);
            }
            let (surface, node) = tab_id.into();
            self.dock_state
                .remove_tab((surface, node, self.tab_count.into()));
        }
    }
}

#[cfg(test)]
mod tests
{
    use dynamo_core::dynamics::DynamicalFamily;

    #[test]
    fn gui_speedtest()
    {
        use dynamo_core::dynamics::julia::JuliaSet;
        use dynamo_gui::interface::{MainInterface, PanePair};

        let height = 1024;
        let parameter_plane = dynamo_profiles::QuadRatPer2::default()
            .with_res_y(height)
            .with_max_iter(2048);

        let dynamical_plane = JuliaSet::from(parameter_plane.clone());

        let mut interface = Box::new(MainInterface::new(parameter_plane, dynamical_plane, height));
        for _ in 0..10 {
            interface.child_mut().recompute();
        }
    }
}
