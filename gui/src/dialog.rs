use egui::{self, Key};
use egui::{vec2, Window};

pub enum DialogState
{
    JustOpened,
    InProgress,
    Completed,
    Closed,
}
impl DialogState
{
    fn visible(&self) -> bool
    {
        match self
        {
            Self::JustOpened | Self::InProgress => true,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct InputDialogBuilder
{
    title: String,
    prompt: String,
}
impl InputDialogBuilder
{
    pub fn title(mut self, title: &str) -> Self
    {
        self.title = title.to_owned();
        self
    }
    pub fn prompt(mut self, prompt: &str) -> Self
    {
        self.prompt = prompt.to_owned();
        self
    }
    pub fn build(self) -> InputDialog
    {
        InputDialog::new(self.title, self.prompt)
    }
}

pub struct InputDialog
{
    pub title: String,
    pub prompt: String,
    pub user_input: String,
    pub state: DialogState,
}

impl InputDialog
{
    #[must_use]
    pub fn new(title: String, prompt: String) -> Self
    {
        Self {
            title,
            prompt,
            user_input: String::new(),
            state: DialogState::JustOpened,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context)
    {
        if self.visible()
        {
            Window::new(self.title.clone())
                .title_bar(false)
                .collapsible(false)
                .fixed_size(vec2(300.0, 100.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&self.prompt);
                        let response = ui.text_edit_singleline(&mut self.user_input);
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    });

                    if ui.button("OK").clicked() || ctx.input(|i| i.key_pressed(Key::Enter))
                    {
                        self.state = DialogState::Completed;
                    }
                    else if ui.button("Cancel").clicked()
                        || ctx.input(|i| i.key_pressed(Key::Escape))
                    {
                        self.disable();
                    }
                });
        }
    }

    #[inline]
    pub fn visible(&self) -> bool
    {
        self.state.visible()
    }

    #[inline]
    pub fn enable(&mut self)
    {
        self.state = DialogState::InProgress;
    }

    #[inline]
    pub fn disable(&mut self)
    {
        self.state = DialogState::Closed;
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
