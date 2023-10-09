use fractal_common::{
    coloring::{algorithms::InteriorColoringAlgorithm, palette::ColorPalette},
    types::{IterCount, Period},
};

use crate::interface::PaneID;

#[derive(Clone, Debug, PartialEq)]
pub enum Action
{
    // UI control
    Quit,
    Close,
    SaveImage(PaneID),
    SaveActiveImage,
    // Annotation toggles
    ToggleSelectionMarker,
    ToggleCritical(PaneID),
    ToggleCycles(PaneID, Period),
    // Dynamics
    FindPeriodicPoint,
    MapSelection,
    DrawOrbit,
    ClearOrbit,
    DrawExternalRay
    {
        select_landing_point: bool,
    },
    DrawEquipotential,
    ClearRays,
    ClearEquipotentials,
    ClearCurves,
    ResetSelection,
    // Image controls
    ToggleLiveMode,
    CycleActivePlane,
    PromptImageHeight,
    Pan(f64, f64),
    Zoom(f64),
    CenterOnSelection,
    ScaleMaxIter(IterCount),
    // Coloring
    RandomizePalette,
    SetPalette(ColorPalette),
    SetPaletteWhite,
    SetPaletteBlack,
    SetColoring(InteriorColoringAlgorithm),
    ScalePalettePeriod(f64),
    ShiftPalettePhase(f64),
}
impl Action
{
    pub fn description(&self) -> String
    {
        match self
        {
            // UI Control
            Action::Quit => "Exit the application.".to_owned(),
            Action::Close => "Close the current tab.".to_owned(),
            Action::SaveImage(pane_id) => format!("Save the {} active image to a file.", pane_id),
            Action::SaveActiveImage => "Save the active image to a file.".to_owned(),

            // Annotation Toggles
            Action::ToggleSelectionMarker => "Toggle selection marker on active image.".to_owned(),
            Action::ToggleCritical(pane_id) =>
            {
                format!("Toggle critical points on {pane_id} image.")
            }
            Action::ToggleCycles(pane_id, period) =>
            {
                format!("Toggle known cycles (or component centers) of period {period} on {pane_id} image.")
            }

            // Dynamics
            Action::FindPeriodicPoint =>
            {
                "Find and select a nearby preperiodic/periodic/pcf point on the active image."
                    .to_owned()
            }
            Action::MapSelection =>
            {
                "Apply dynamical map to current selection on dynamical plane.".to_owned()
            }
            Action::DrawOrbit =>
            {
                "Draw the orbit of currently selected point on dynamical plane.".to_owned()
            }
            Action::ClearOrbit => "Hide orbit from active plane.".to_owned(),
            Action::DrawExternalRay {
                select_landing_point,
            } =>
            {
                if *select_landing_point
                {
                    "Draw/hide an external ray and select its landing point on active image."
                        .to_owned()
                }
                else
                {
                    "Draw/hide an external ray on active image.".to_owned()
                }
            }
            Action::DrawEquipotential => "Draw equipotential through selection.".to_owned(),
            Action::ClearRays => "Clear all external rays on active image.".to_owned(),
            Action::ClearEquipotentials => "Clear all equipotentials on active image.".to_owned(),
            Action::ClearCurves => "Clear all curves on active image.".to_owned(),
            Action::ResetSelection => "Reset selection to default on active image.".to_owned(),

            // Image Controls
            Action::ToggleLiveMode =>
            {
                "Toggle \"live Julia mode\", in which child plane changes with cursor movement."
                    .to_owned()
            }
            Action::CycleActivePlane => "Cycle through different planes of the fractal.".to_owned(),
            Action::PromptImageHeight =>
            {
                "Prompt to set the height of the fractal image.".to_owned()
            }
            Action::Pan(x, y) =>
            {
                if *x == 0.
                {
                    if *y > 0.
                    {
                        format!("Pan upw by {}%", y * 100.)
                    }
                    else
                    {
                        format!("Pan down by {}%", y * 100.)
                    }
                }
                else if *y == 0.
                {
                    if *x > 0.
                    {
                        format!("Pan right by {}%", y * 100.)
                    }
                    else
                    {
                        format!("Pan left by {}%", y * 100.)
                    }
                }
                else
                {
                    format!("Pan the view (x: {x}, y: {y}))")
                }
            }
            Action::Zoom(scale) =>
            {
                format!("Zoom {} (scale: {:.2})", in_or_out(*scale), *scale)
            }
            Action::CenterOnSelection => "Center view on selected point.".to_owned(),
            Action::ScaleMaxIter(scale) =>
            {
                format!(
                    "{} max iterations on active image (factor: {scale})",
                    inc_or_dec(*scale)
                )
            }

            // Coloring
            Action::RandomizePalette => "Randomize the color palette.".to_owned(),
            Action::SetPalette(_) => "Set the color palette.".to_owned(),
            Action::SetPaletteWhite => "Use black on white palette.".to_owned(),
            Action::SetPaletteBlack => "Use white on black palette.".to_owned(),
            Action::SetColoring(algorithm) =>
            {
                use InteriorColoringAlgorithm::*;
                let desc = match algorithm
                {
                    Solid => "Color bounded components black.",
                    Period => "Color bounded components by period",
                    PeriodMultiplier => "Color bounded components by period and norm of multiplier",
                    Multiplier => "Color bounded components by multiplier",
                    Preperiod => "Color bounded components by convergence time",
                    InternalPotential { .. } =>
                    {
                        "Color bounded components by internal potential (Kœnigs or Böttcher map)"
                    }
                    PreperiodPeriod => "Color bounded components by period and convergence time",
                    PreperiodPeriodSmooth { .. } =>
                    {
                        "Color bounded components by period and internal potential"
                    }
                };
                desc.to_owned()
            }
            Action::ScalePalettePeriod(scale) =>
            {
                format!("{} the period of the color palette.", inc_or_dec(*scale))
            }
            Action::ShiftPalettePhase(_) => "Shift the phase of the color palette.".to_owned(),
        }
    }

    pub fn short_description(&self) -> String
    {
        match self
        {
            // UI Control
            Action::Quit => "Exit".to_owned(),
            Action::Close => "Close Tab".to_owned(),
            Action::SaveActiveImage => "Save Image".to_owned(),
            Action::SaveImage(pane_id) => format!("Save {:#}", pane_id),

            // Annotation Toggles
            Action::ToggleSelectionMarker => "Toggle Selection".to_owned(),
            Action::ToggleCritical(pane_id) => match pane_id
            {
                PaneID::Parent => "Toggle marked pts (parent)".to_owned(),
                PaneID::Child => "Toggle Critical".to_owned(),
            },
            Action::ToggleCycles(_, p) => format!("Toggle {p}-cycles"),

            // Dynamics
            Action::FindPeriodicPoint => "Find Point".to_owned(),
            Action::MapSelection => "Map Selection".to_owned(),
            Action::DrawOrbit => "Draw Orbit".to_owned(),
            Action::ClearOrbit => "Clear Orbit".to_owned(),
            Action::DrawExternalRay {
                select_landing_point,
            } =>
            {
                if *select_landing_point
                {
                    "Ray to Point".to_owned()
                }
                else
                {
                    "Draw Ray".to_owned()
                }
            }
            Action::DrawEquipotential => "Equipotential".to_owned(),
            Action::ClearRays => "Clear Rays".to_owned(),
            Action::ClearEquipotentials => "Clear Equipotentials".to_owned(),
            Action::ClearCurves => "Clear Curves".to_owned(),
            Action::ResetSelection => "Reset Selection".to_owned(),

            // Image Controls
            Action::ToggleLiveMode => "Toggle Live Mode".to_owned(),
            Action::CycleActivePlane => "Cycle Plane".to_owned(),
            Action::PromptImageHeight => "Set Height".to_owned(),
            Action::Pan(_, _) => "Pan View".to_owned(),
            Action::Zoom(scale) => format!("Zoom {}", in_or_out(*scale)),
            Action::CenterOnSelection => "Center View".to_owned(),
            Action::ScaleMaxIter(scale) => format!("{} iters", inc_or_dec(*scale)),

            // Coloring
            Action::RandomizePalette => "Random".to_owned(),
            Action::SetPalette(_) => "Custom".to_owned(),
            Action::SetPaletteWhite => "White".to_owned(),
            Action::SetPaletteBlack => "Black".to_owned(),
            Action::SetColoring(algorithm) =>
            {
                use InteriorColoringAlgorithm::*;
                let desc = match algorithm
                {
                    Solid => "Black",
                    Period => "Period",
                    PeriodMultiplier => "Period + Multiplier",
                    Multiplier => "Multiplier",
                    Preperiod => "Convergence time",
                    InternalPotential { .. } => "Internal Potential",
                    PreperiodPeriod => "Period + Conv. Time",
                    PreperiodPeriodSmooth { .. } => "Period + Potential",
                };
                desc.to_owned()
            }
            Action::ScalePalettePeriod(scale) => format!("{} density", inc_or_dec(1.0 / scale)),
            Action::ShiftPalettePhase(_) => "Adjust Phase".to_owned(),
        }
    }
}

fn in_or_out(scale: f64) -> String
{
    if scale < 0.5
    {
        "in far".to_owned()
    }
    else if scale <= 1.
    {
        "in".to_owned()
    }
    else if scale < 2.
    {
        "out".to_owned()
    }
    else
    {
        "out far".to_owned()
    }
}

fn inc_or_dec(scale: f64) -> String
{
    if scale < 1.0
    {
        "Decrease".to_owned()
    }
    else
    {
        "Increase".to_owned()
    }
}
