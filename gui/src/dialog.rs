use egui;
use egui::{vec2, Window};

pub struct InputDialog
{
    pub title: String,
    pub prompt: String,
    pub user_input: String,
    visible: bool,
    on_ok: Box<dyn FnMut(String)>,
}

impl InputDialog
{
    #[must_use]
    pub fn new<F: FnMut(String) + 'static>(title: String, prompt: String, on_ok: F) -> Self
    {
        Self {
            title,
            prompt,
            user_input: String::new(),
            visible: false,
            on_ok: Box::new(on_ok),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible
        {
            Window::new(self.title.clone())
                .title_bar(false)
                .collapsible(false)
                .fixed_size(vec2(300.0, 100.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.prompt);
                        ui.text_edit_singleline(&mut self.user_input);
                    });

                    if ui.button("OK").clicked()
                    {
                        self.visible = false;
                        (self.on_ok)(self.user_input.clone());
                    }
                    else if ui.button("Cancel").clicked()
                    {
                        self.disable();
                    }
                });
        }
    }

    #[inline]
    pub fn visible(&self) -> bool
    {
        self.visible
    }

    #[inline]
    fn hide(&mut self)
    {
        self.visible = false;
    }

    #[inline]
    pub fn enable(&mut self)
    {
        self.visible = true
    }

    #[inline]
    pub fn disable(&mut self)
    {
        self.hide();
        self.reset_input();
    }

    #[inline]
    pub fn set_input(&mut self, user_input: String)
    {
        self.user_input = user_input;
    }

    #[inline]
    pub fn get_input(&self) -> &str
    {
        &self.user_input
    }

    #[inline]
    pub fn reset_input(&mut self)
    {
        self.user_input = "".to_owned();
    }
}
