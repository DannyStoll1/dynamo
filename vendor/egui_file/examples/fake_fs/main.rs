use eframe::{
  App, Frame,
  egui::{CentralPanel, Context},
};
use egui_file::FileDialog;
use std::path::PathBuf;

mod fake_fs;

#[derive(Default)]
pub struct Demo {
  opened_file: Option<PathBuf>,
  open_file_dialog: Option<FileDialog>,
}

impl App for Demo {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    CentralPanel::default().show(ctx, |ui| {
      if (ui.button("Open")).clicked() {
        let mut dialog = FileDialog::open_file()
          .with_fs(Box::new(fake_fs::FakeFs::new()))
          .initial_path("/");
        dialog.open();
        self.open_file_dialog = Some(dialog);
      }

      if let Some(dialog) = &mut self.open_file_dialog {
        if dialog.show(ctx).selected() {
          if let Some(file) = dialog.path() {
            self.opened_file = Some(file.to_path_buf());
          }
        }
      }
    });
  }
}

fn main() {
  let _ = eframe::run_native(
    "VFS File Dialog Demo",
    eframe::NativeOptions::default(),
    Box::new(|_cc| Ok(Box::new(Demo::default()))),
  );
}
