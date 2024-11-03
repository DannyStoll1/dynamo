use directories::{ProjectDirs, UserDirs};
use std::path::PathBuf;

pub fn images_dir() -> Option<PathBuf>
{
    let user_dirs = UserDirs::new()?;
    let pictures = user_dirs
        .picture_dir()
        .map_or_else(|| user_dirs.home_dir().join("Pictures"), ToOwned::to_owned);
    let dynamo_images = pictures.join("Dynamo");
    std::fs::create_dir_all(&dynamo_images).ok()?;
    Some(dynamo_images)
}

#[must_use]
pub fn palettes_dir() -> Option<PathBuf>
{
    let proj_dirs = ProjectDirs::from("com", "Zero Ideal", "Dynamo")?;
    let palettes_dir = proj_dirs.data_dir().join("palettes");
    std::fs::create_dir_all(&palettes_dir).ok()?;
    Some(palettes_dir)
}

#[must_use]
pub fn script_dir() -> Option<PathBuf>
{
    let proj_dirs = ProjectDirs::from("com", "Zero Ideal", "Dynamo")?;
    let scripts_dir = proj_dirs.data_dir().join("scripts");
    std::fs::create_dir_all(&scripts_dir).ok()?;
    Some(scripts_dir)
}
