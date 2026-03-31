use eframe::{
  App, Frame,
  egui::{CentralPanel, Context},
};
use egui_file::FileDialog;
use std::{
  ffi::OsStr,
  path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Demo {
  opened_file: Option<PathBuf>,
  open_file_dialog: Option<FileDialog>,
}

impl App for Demo {
  fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
    CentralPanel::default().show(ctx, |ui| {
      if (ui.button("Open")).clicked() {
        // Show only Rust source files.
        let filter = Box::new({
          let ext = Some(OsStr::new("rs"));
          move |path: &Path| -> bool { path.extension() == ext }
        });

        let mut dialog = FileDialog::open_file().show_files_filter(filter);
        if let Some(mut path) = self.opened_file.clone()
          && path.pop()
        {
          // Use the path of the previously opened file.
          dialog = dialog.initial_path(path);
        }

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
    "Open File Demo",
    eframe::NativeOptions::default(),
    Box::new(|_cc| Ok(Box::new(Demo::default()))),
  );
}
