#![allow(dead_code)]

use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

pub mod dialog;
pub mod fractal_tab;
pub mod image_frame;
pub mod keyboard_shortcuts;
pub mod macros;
pub mod marked_points;
pub mod pane;
// mod utils;
use fractal_tab::FractalTab;

#[cfg(not(target_arch = "wasm32"))]
pub fn run_app() -> Result<(), eframe::Error>
{
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Run complex dynamics simulations",
        options,
        Box::new(|_cc| Box::<FractalApp>::default()),
    )
}

struct TabViewer<'a>
{
    added_nodes: &'a mut Vec<FractalTab>,
}

impl egui_dock::TabViewer for TabViewer<'_>
{
    type Tab = FractalTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab)
    {
        ui.label(tab.interface.name());
        tab.show_menu(ui);
        tab.interface.handle_input(ui.ctx());
        tab.process_interface_message(ui);
        tab.interface.process_tasks();
        tab.interface.show_dialogs(ui.ctx());
        tab.interface.show(ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText
    {
        format!("Tab {}", tab.interface.name()).into()
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex)
    {
        let tab = FractalTab::default().with_surface_and_node_index(surface, node);
        self.added_nodes.push(tab);
    }

    // fn add_popup(&mut self, ui: &mut egui::Ui, node: NodeIndex) {
    //     ui.set_min_width(120.0);
    //     ui.style_mut().visuals.button_frame = false;
    //
    //     if ui.button("Mandelbrot").clicked() {
    //         let tab = FractalTab::default().with_node_index(node);
    //         self.added_nodes.push(tab);
    //     }
    //
    //     if ui.button("QuadRatPer2").clicked() {
    //         let tab = FractalTab::default().with_node_index(node);
    //         self.added_nodes.push(tab);
    //     }
    // }
}

pub struct FractalApp
{
    dock_state: DockState<FractalTab>,
    counter: usize,
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let tab0 = FractalTab::default();

        let dock_state = DockState::new(vec![tab0]);

        // You can modify the tree before constructing the dock
        // let [_, _] = tree.split_right(NodeIndex::root(), 0.5, vec![1]);
        // let [_, _] = tree.split_below(a, 0.7, vec![4]);
        // let [_, _] = tree.split_below(b, 0.5, vec![5]);

        Self {
            dock_state,
            counter: 1,
        }
    }
}

impl eframe::App for FractalApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        let mut added_nodes = Vec::new();
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
                },
            );
        for tab in added_nodes
        {
            self.dock_state
                .set_focused_node_and_surface((tab.surface, tab.node));
            // self.tree.push_to_focused_leaf(FractalTab {
            //     interface: tab.interface,
            //     node: NodeIndex(self.counter),
            // });
            self.dock_state.push_to_focused_leaf(FractalTab::default());
            self.counter += 1;
        }
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn gui_speedtest()
    {
        use fractal_core::dynamics::{julia::JuliaSet, ParameterPlane};
        let height = 1024;
        use crate::pane::{MainInterface, PanePair};
        let parameter_plane = fractal_profiles::QuadRatPer2::default()
            .with_res_y(height)
            .with_max_iter(2048);

        let dynamical_plane = JuliaSet::from(parameter_plane.clone());

        let mut pane_pair = Box::new(MainInterface::new(parameter_plane, dynamical_plane, height));
        for _ in 0..10
        {
            pane_pair.child_mut().recompute();
        }
    }
}
