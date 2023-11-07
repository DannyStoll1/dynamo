use crate::pane::id::*;
use dynamo_color::{IncoloringAlgorithm, Palette};
use dynamo_common::types::{IterCount, Period};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action
{
    // UI control
    Quit,
    Close,
    NewTab,
    SaveImage(PaneSelection),
    SavePalette(PaneSelection),
    LoadPalette(PaneSelection),
    // Annotation toggles
    ToggleSelectionMarker,
    ToggleCritical(PaneSelection),
    ToggleCycles(PaneSelection, Period),
    // Dynamics
    FindPeriodicPoint,
    MapSelection,
    EnterCoordinates,
    DrawOrbit,
    ClearOrbit,
    DrawExternalRay
    {
        select_landing_point: bool,
    },
    DrawRaysOfPeriod,
    DrawEquipotential,
    ClearRays,
    ClearEquipotentials,
    ClearCurves,
    ResetSelection,
    ResetView,
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
    SetPalette(Palette),
    SetPaletteWhite,
    SetPaletteBlack,
    SetColoring(IncoloringAlgorithm),
    SetColoringInternalPotential,
    SetColoringPotentialPeriod,
    SetColoringPreperiodPeriod,
    ScalePalettePeriod(f64),
    ShiftPalettePhase(f64),
}
impl Action
{
    #[must_use]
    pub fn description(&self) -> String
    {
        match self {
            // UI Control
            Self::Quit => "Exit the application.".to_owned(),
            Self::Close => "Close the current tab.".to_owned(),
            Self::NewTab => "Open a new tab.".to_owned(),
            Self::SaveImage(pane_id) => format!("Save the {pane_id} image to a file."),
            Self::SavePalette(pane_id) => format!("Save the {pane_id} palette to a file."),
            Self::LoadPalette(pane_id) => format!("Load palette for {pane_id} from file"),

            // Annotation Toggles
            Self::ToggleSelectionMarker => "Toggle selection marker on active image.".to_owned(),
            Self::ToggleCritical(pane_id) => {
                format!("Toggle critical points on {pane_id} image.")
            }
            Self::ToggleCycles(pane_id, period) => {
                format!("Toggle known cycles (or component centers) of period {period} on {pane_id} image.")
            }

            // Dynamics
            Self::FindPeriodicPoint => {
                "Find and select a nearby preperiodic/periodic/pcf point on the active image."
                    .to_owned()
            }
            Self::EnterCoordinates => {
                "Enter coordinates to select a point on active image.".to_owned()
            }
            Self::MapSelection => {
                "Apply dynamical map to current selection on dynamical plane.".to_owned()
            }
            Self::DrawOrbit => {
                "Draw the orbit of currently selected point on dynamical plane.".to_owned()
            }
            Self::ClearOrbit => "Hide orbit from dynamical plane.".to_owned(),
            Self::DrawExternalRay {
                select_landing_point,
            } => {
                if *select_landing_point {
                    "Draw/hide an external ray and select its landing point on active image."
                        .to_owned()
                } else {
                    "Draw/hide an external ray on active image.".to_owned()
                }
            }
            Self::DrawRaysOfPeriod => "Draw all rays of a given period and preperiod.".to_owned(),
            Self::DrawEquipotential => "Draw equipotential through selection.".to_owned(),
            Self::ClearRays => "Clear all external rays on active image.".to_owned(),
            Self::ClearEquipotentials => "Clear all equipotentials on active image.".to_owned(),
            Self::ClearCurves => "Clear all curves on active image.".to_owned(),
            Self::ResetSelection => "Reset selection to default on active image.".to_owned(),
            Self::ResetView => "Reset bounds and selection to default on active image.".to_owned(),

            // Image Controls
            Self::ToggleLiveMode => {
                "Toggle \"live Julia mode\", in which child plane changes with cursor movement."
                    .to_owned()
            }
            Self::CycleActivePlane => "Cycle through different planes of the fractal.".to_owned(),
            Self::PromptImageHeight => "Prompt to set the height of the fractal image.".to_owned(),
            Self::Pan(x, y) => {
                if *x == 0. {
                    if *y > 0. {
                        format!("Pan upw by {}%", y * 100.)
                    } else {
                        format!("Pan down by {}%", y * 100.)
                    }
                } else if *y == 0. {
                    if *x > 0. {
                        format!("Pan right by {}%", y * 100.)
                    } else {
                        format!("Pan left by {}%", y * 100.)
                    }
                } else {
                    format!("Pan the view (x: {x}, y: {y}))")
                }
            }
            Self::Zoom(scale) => {
                format!("Zoom {} (scale: {:.2})", in_or_out(*scale), *scale)
            }
            Self::CenterOnSelection => "Center view on selected point.".to_owned(),
            Self::ScaleMaxIter(scale) => {
                format!(
                    "{} max iterations on active image (factor: {scale})",
                    inc_or_dec(*scale)
                )
            }

            // Coloring
            Self::RandomizePalette => "Randomize the color palette.".to_owned(),
            Self::SetPalette(_) => "Set the color palette.".to_owned(),
            Self::SetPaletteWhite => "Use black on white palette.".to_owned(),
            Self::SetPaletteBlack => "Use white on black palette.".to_owned(),
            Self::SetColoring(algorithm) => {
                use IncoloringAlgorithm::*;
                let desc = match algorithm {
                    Solid => "Color bounded components black.",
                    Period => "Color bounded components by period",
                    PeriodMultiplier => "Color bounded components by period and norm of multiplier",
                    Multiplier => "Color bounded components by multiplier",
                    Preperiod => "Color bounded components by convergence time",
                    InternalPotential { .. } => {
                        "Color bounded components by internal potential (Kœnigs or Böttcher map)"
                    }
                    PreperiodPeriod { .. } => {
                        "Color bounded components by period and convergence time"
                    }
                    PotentialAndPeriod { .. } => {
                        "Color bounded components by period and internal potential"
                    }
                };
                desc.to_owned()
            }
            Self::SetColoringInternalPotential => {
                "Color bounded components by internal potential (Kœnigs or Böttcher map)".to_owned()
            }
            Self::SetColoringPotentialPeriod => {
                "Color bounded components by period and internal potential".to_owned()
            }
            Self::SetColoringPreperiodPeriod => {
                "Color bounded components by period and convergence time".to_owned()
            }
            Self::ScalePalettePeriod(scale) => {
                format!("{} the period of the color palette.", inc_or_dec(*scale))
            }
            Self::ShiftPalettePhase(_) => "Shift the phase of the color palette.".to_owned(),
        }
    }

    #[must_use]
    pub fn short_description(&self) -> String
    {
        match self {
            // UI Control
            Self::Quit => "Exit".to_owned(),
            Self::Close => "Close Tab".to_owned(),
            Self::NewTab => "New Tab".to_owned(),
            Self::SaveImage(pane_selection) => format!("Save{pane_selection:#}..."),
            Self::SavePalette(pane_selection) => format!("Save{pane_selection:#} Palette..."),
            Self::LoadPalette(pane_selection) => format!("Load{pane_selection:#} Palette..."),

            // Annotation Toggles
            Self::ToggleSelectionMarker => "Toggle Selection".to_owned(),
            Self::ToggleCritical(pane_id) => match pane_id {
                PaneSelection::ActivePane => "Toggle marked pts (active pane)".to_owned(),
                PaneSelection::BothPanes => "Toggle marked pts".to_owned(),
                PaneSelection::Id(PaneID::Parent) => "Toggle marked pts (parent)".to_owned(),
                PaneSelection::Id(PaneID::Child) => "Toggle Critical".to_owned(),
            },
            Self::ToggleCycles(_, p) => format!("Toggle {p}-cycles"),

            // Dynamics
            Self::FindPeriodicPoint => "Find Point...".to_owned(),
            Self::EnterCoordinates => "Enter Point...".to_owned(),
            Self::MapSelection => "Map Selection".to_owned(),
            Self::DrawOrbit => "Draw Orbit".to_owned(),
            Self::ClearOrbit => "Clear Orbit".to_owned(),
            Self::DrawExternalRay {
                select_landing_point,
            } => {
                if *select_landing_point {
                    "Ray to Point...".to_owned()
                } else {
                    "Draw Ray...".to_owned()
                }
            }
            Self::DrawRaysOfPeriod => "Rays of Period".to_owned(),
            Self::DrawEquipotential => "Equipotential".to_owned(),
            Self::ClearRays => "Clear Rays".to_owned(),
            Self::ClearEquipotentials => "Clear Equipotentials".to_owned(),
            Self::ClearCurves => "Clear Curves".to_owned(),
            Self::ResetSelection => "Reset Selection".to_owned(),
            Self::ResetView => "Reset View".to_owned(),

            // Image Controls
            Self::ToggleLiveMode => "Toggle Live Mode".to_owned(),
            Self::CycleActivePlane => "Cycle Plane".to_owned(),
            Self::PromptImageHeight => "Set Height".to_owned(),
            Self::Pan(_, _) => "Pan View".to_owned(),
            Self::Zoom(scale) => format!("Zoom {}", in_or_out(*scale)),
            Self::CenterOnSelection => "Center View".to_owned(),
            Self::ScaleMaxIter(scale) => format!("{} iters", inc_or_dec(*scale)),

            // Coloring
            Self::RandomizePalette => "Random".to_owned(),
            Self::SetPalette(_) => "Custom".to_owned(),
            Self::SetPaletteWhite => "White".to_owned(),
            Self::SetPaletteBlack => "Black".to_owned(),
            Self::SetColoring(algorithm) => {
                use IncoloringAlgorithm::*;
                let desc = match algorithm {
                    Solid => "Black",
                    Period => "Period",
                    PeriodMultiplier => "Period + Multiplier",
                    Multiplier => "Multiplier",
                    Preperiod => "Convergence time",
                    InternalPotential { .. } => "Internal Potential",
                    PreperiodPeriod { .. } => "Period + Conv. Time",
                    PotentialAndPeriod { .. } => "Period + Potential",
                };
                desc.to_owned()
            }
            Self::SetColoringInternalPotential => "Internal Potential".to_owned(),
            Self::SetColoringPotentialPeriod => "Period + Potential".to_owned(),
            Self::SetColoringPreperiodPeriod => "Period + Conv. Time".to_owned(),
            Self::ScalePalettePeriod(scale) => format!("{} density", inc_or_dec(1.0 / scale)),
            Self::ShiftPalettePhase(_) => "Adjust Phase".to_owned(),
        }
    }
}

fn in_or_out(scale: f64) -> String
{
    if scale < 0.5 {
        "in far".to_owned()
    } else if scale <= 1. {
        "in".to_owned()
    } else if scale < 2. {
        "out".to_owned()
    } else {
        "out far".to_owned()
    }
}

fn inc_or_dec(scale: f64) -> String
{
    if scale < 1.0 {
        "Decrease".to_owned()
    } else {
        "Increase".to_owned()
    }
}
