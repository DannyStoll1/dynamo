use crate::coloring::{algorithms::ColoringAlgorithm, palette::ColorPalette};
use crate::dynamics::{covering_maps::HasDynamicalCovers, julia::JuliaSet, ParameterPlane};
use crate::profiles::*;
use crate::types::{ParamList, Period};

use eframe::App;
use egui;
use egui_dock::{DockArea, NodeIndex, Style, Tree};

pub mod fractal_tab;
pub mod image_frame;
pub mod keyboard_shortcuts;
pub mod marked_points;
pub mod pane;
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
        tab.interface.show_save_dialog(ui.ctx());
        tab.interface.show(ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText
    {
        format!("Tab {}", tab.interface.name()).into()
    }

    fn on_add(&mut self, node: NodeIndex)
    {
        let tab = FractalTab::default().with_node_index(node);
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
    tree: Tree<FractalTab>,
    counter: usize,
}

impl Default for FractalApp
{
    fn default() -> Self
    {
        let tab0 = FractalTab::default();

        let tree = Tree::new(vec![tab0]);

        // You can modify the tree before constructing the dock
        // let [_, _] = tree.split_right(NodeIndex::root(), 0.5, vec![1]);
        // let [_, _] = tree.split_below(a, 0.7, vec![4]);
        // let [_, _] = tree.split_below(b, 0.5, vec![5]);

        Self { tree, counter: 1 }
    }
}

impl eframe::App for FractalApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .show_add_buttons(true)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tabs.fill_tab_bar = true;
                style
            })
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                },
            );
        added_nodes.into_iter().for_each(|tab| {
            self.tree.set_focused_node(tab.node);
            // self.tree.push_to_focused_leaf(FractalTab {
            //     interface: tab.interface,
            //     node: NodeIndex(self.counter),
            // });
            self.tree.push_to_focused_leaf(FractalTab::default());
            self.counter += 1;
        });
    }
}
