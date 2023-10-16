use crate::interface::{Interactive, Interface, UIMessage};
use egui::Ui;
use libloading::Library;

pub struct InterfaceHolder<'i>
{
    pub interface: Box<dyn Interface + 'i>,
    _library: Library,
}
impl<'i> InterfaceHolder<'i>
{
    pub fn new(interface: Box<dyn Interface>, library: Library) -> Self
    {
        Self {
            interface,
            _library: library,
        }
    }
}

impl<'i> Interactive for InterfaceHolder<'i>
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
    fn get_message(&self) -> UIMessage
    {
        self.interface.get_message()
    }
    fn pop_message(&mut self) -> UIMessage
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

impl<'i> Interface for InterfaceHolder<'i>
{
    fn update(&mut self, ui: &mut Ui)
    {
        self.interface.update(ui)
    }
}
