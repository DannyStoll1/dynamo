use egui::{Context, Ui};
use libloading::Library;

use crate::interface::{Interactive, Interface, UiMessage};

pub struct InterfaceHolder<'i>
{
    pub interface: Box<dyn Interface + 'i>,
    _library:      Library,
}
impl InterfaceHolder<'_>
{
    #[must_use]
    pub fn new(interface: Box<dyn Interface>, library: Library) -> Self
    {
        Self {
            interface,
            _library: library,
        }
    }
}

impl Interactive for InterfaceHolder<'_>
{
    fn name(&self) -> String
    {
        self.interface.name()
    }
    fn show(&mut self, ui: &mut Ui)
    {
        self.interface.show(ui);
    }
    fn show_dialog(&mut self, ctx: &egui::Context)
    {
        self.interface.show_dialog(ctx);
    }
    fn reset_click(&mut self)
    {
        self.interface.reset_click();
    }
    fn get_message(&self) -> UiMessage
    {
        self.interface.get_message()
    }
    fn pop_message(&mut self) -> UiMessage
    {
        self.interface.pop_message()
    }
    fn handle_input(&mut self, ctx: &egui::Context)
    {
        self.interface.handle_input(ctx);
    }
    fn consume_click(&mut self)
    {
        self.interface.consume_click();
    }
    fn change_height(&mut self, new_height: usize)
    {
        self.interface.change_height(new_height);
    }
    fn get_image_height(&self) -> usize
    {
        self.interface.get_image_height()
    }
    fn process_action(&mut self, action: &crate::actions::Action)
    {
        self.interface.process_action(action);
    }
}

impl Interface for InterfaceHolder<'_>
{
    fn update(&mut self, ctx: &Context)
    {
        self.interface.update(ctx);
    }
}
