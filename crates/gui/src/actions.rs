use dynamo_color::{IncoloringAlgorithm, Palette};
use dynamo_common::types::{IterCountSmooth, Period};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::marked_points::ContourType;
use crate::pane::id::PaneSelection;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
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
    ToggleCritical,
    ToggleMarked(PaneSelection),
    ToggleCycles(PaneSelection, Period),
    // Dynamics
    FindPeriodicPoint,
    MapSelection,
    EnterCoordinates,
    DrawOrbit,
    ClearOrbit,
    DrawExternalRay
    {
        include_orbit:        bool,
        select_landing_point: bool,
    },
    DrawRaysOfPeriod,
    DrawContour(ContourType),
    DrawAuxContours,
    ClearRays,
    ClearEquipotentials,
    ClearCurves,
    StopFollowing,
    ResetSelection,
    ResetView,
    // Image controls
    ToggleLiveMode,
    CycleActivePlane,
    PromptImageHeight,
    Pan(f64, f64),
    Zoom(f64),
    CenterOnSelection,
    ScaleMaxIter(IterCountSmooth),
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
    ToggleEscapePhaseColoring,
    CycleComputeMode(PaneSelection, ChangeBoolean),
}
impl Action
{
    #[must_use]
    pub fn description(&self) -> String
    {
        self.text(DescriptionKind::Long)
    }

    #[must_use]
    pub fn short_description(&self) -> String
    {
        self.text(DescriptionKind::Short)
    }

    fn text(&self, kind: DescriptionKind) -> String
    {
        self.ui_text(kind)
            .or_else(|| self.annotation_text(kind))
            .or_else(|| self.dynamics_text(kind))
            .or_else(|| self.image_text(kind))
            .or_else(|| self.coloring_text(kind))
            .unwrap_or_default()
    }

    fn ui_text(&self, kind: DescriptionKind) -> Option<String>
    {
        Some(match self {
            Self::Quit => kind.label("Exit the application.", "Exit"),
            Self::Close => kind.label("Close the current tab.", "Close Tab"),
            Self::NewTab => kind.label("Open a new tab.", "New Tab"),
            Self::SaveImage(pane_id) => kind.save_image(*pane_id),
            Self::SavePalette(pane_id) => kind.save_palette(*pane_id),
            Self::LoadPalette(pane_id) => kind.load_palette(*pane_id),
            _ => return None,
        })
    }

    fn annotation_text(&self, kind: DescriptionKind) -> Option<String>
    {
        Some(match self {
            Self::ToggleSelectionMarker => kind.label(
                "Toggle selection marker on active image.",
                "Toggle Selection",
            ),
            Self::ToggleCritical => kind.label(
                "Toggle critical points on dynamical plane.",
                "Toggle Critical",
            ),
            Self::ToggleMarked(pane_id) => kind.toggle_marked(*pane_id),
            Self::ToggleCycles(pane_id, period) => kind.toggle_cycles(*pane_id, *period),
            _ => return None,
        })
    }

    fn dynamics_text(&self, kind: DescriptionKind) -> Option<String>
    {
        Some(match self {
            Self::FindPeriodicPoint => kind.label(
                "Find and select a nearby preperiodic/periodic/pcf point on the active image.",
                "Find Point...",
            ),
            Self::EnterCoordinates => kind.label(
                "Enter coordinates to select a point on active image.",
                "Enter Point...",
            ),
            Self::MapSelection => kind.label(
                "Apply dynamical map to current selection on dynamical plane.",
                "Map Selection",
            ),
            Self::DrawOrbit => kind.label(
                "Draw the orbit of currently selected point on dynamical plane.",
                "Draw Orbit",
            ),
            Self::ClearOrbit => kind.label("Hide orbit from dynamical plane.", "Clear Orbit"),
            Self::DrawExternalRay {
                include_orbit,
                select_landing_point,
            } => kind.draw_external_ray(*include_orbit, *select_landing_point),
            Self::DrawRaysOfPeriod => kind.label(
                "Draw all rays of a given period and preperiod.",
                "Rays of Period",
            ),
            Self::DrawContour(contour_type) => kind.draw_contour(*contour_type),
            Self::DrawAuxContours => kind.label(
                "Draw contours for the multiplier map a dynamical variety.",
                "Multiplier Contours",
            ),
            Self::ClearRays => kind.label("Clear all external rays on active image.", "Clear Rays"),
            Self::ClearEquipotentials => kind.label(
                "Clear all equipotentials on active image.",
                "Clear Equipotentials",
            ),
            Self::ClearCurves => kind.label("Clear all curves on active image.", "Clear Curves"),
            Self::StopFollowing => kind.label("Stop following points around.", "Stop Following"),
            Self::ResetSelection => kind.label(
                "Reset selection to default on active image.",
                "Reset Selection",
            ),
            Self::ResetView => kind.label(
                "Reset bounds and selection to default on active image.",
                "Reset View",
            ),
            _ => return None,
        })
    }

    fn image_text(&self, kind: DescriptionKind) -> Option<String>
    {
        Some(match self {
            Self::ToggleLiveMode => kind.label(
                "Toggle \"live Julia mode\", in which child plane changes with cursor movement.",
                "Toggle Live Mode",
            ),
            Self::CycleActivePlane => kind.label(
                "Cycle through different planes of the fractal.",
                "Cycle Plane",
            ),
            Self::PromptImageHeight => kind.label(
                "Prompt to set the height of the fractal image.",
                "Set Height",
            ),
            Self::Pan(x, y) => kind.pan(*x, *y),
            Self::Zoom(scale) => kind.zoom(*scale),
            Self::CenterOnSelection => kind.label("Center view on selected point.", "Center View"),
            Self::ScaleMaxIter(scale) => kind.scale_max_iter(*scale),
            _ => return None,
        })
    }

    fn coloring_text(&self, kind: DescriptionKind) -> Option<String>
    {
        Some(match self {
            Self::RandomizePalette => kind.label("Randomize the color palette.", "Random"),
            Self::SetPalette(_) => kind.label("Set the color palette.", "Custom"),
            Self::SetPaletteWhite => kind.label("Use black on white palette.", "White"),
            Self::SetPaletteBlack => kind.label("Use white on black palette.", "Black"),
            Self::SetColoring(algorithm) => kind.set_coloring(algorithm),
            Self::SetColoringInternalPotential => kind.label(
                "Color bounded components by internal potential (Kœnigs or Böttcher map)",
                "Internal Potential",
            ),
            Self::SetColoringPotentialPeriod => kind.label(
                "Color bounded components by period and internal potential",
                "Period + Potential",
            ),
            Self::SetColoringPreperiodPeriod => kind.label(
                "Color bounded components by period and convergence time",
                "Period + Conv. Time",
            ),
            Self::ScalePalettePeriod(scale) => kind.scale_palette_period(*scale),
            Self::ShiftPalettePhase(_) => {
                kind.label("Shift the phase of the color palette.", "Adjust Phase")
            }
            Self::ToggleEscapePhaseColoring => kind.label(
                "Toggle coloring based on phase at time of escape.",
                "Phase Coloring",
            ),
            Self::CycleComputeMode(_, change) => kind.compute_mode(*change),
            _ => return None,
        })
    }
}

#[derive(Clone, Copy)]
enum DescriptionKind
{
    Long,
    Short,
}

impl DescriptionKind
{
    fn label(self, long: &str, short: &str) -> String
    {
        match self {
            Self::Long => long.to_owned(),
            Self::Short => short.to_owned(),
        }
    }

    fn save_image(self, pane: PaneSelection) -> String
    {
        match self {
            Self::Long => format!("Save the {pane} image to a file."),
            Self::Short => format!("Save{pane:#}..."),
        }
    }

    fn save_palette(self, pane: PaneSelection) -> String
    {
        match self {
            Self::Long => format!("Save the {pane} palette to a file."),
            Self::Short => format!("Save{pane:#} Palette..."),
        }
    }

    fn load_palette(self, pane: PaneSelection) -> String
    {
        match self {
            Self::Long => format!("Load palette for {pane} from file"),
            Self::Short => format!("Load{pane:#} Palette..."),
        }
    }

    fn toggle_marked(self, pane: PaneSelection) -> String
    {
        match self {
            Self::Long => format!("Toggle marked points on {pane} image."),
            Self::Short => "Toggle Marked pts".to_owned(),
        }
    }

    fn toggle_cycles(self, pane: PaneSelection, period: Period) -> String
    {
        match self {
            Self::Long => {
                format!(
                    "Toggle known cycles (or component centers) of period {period} on {pane} image."
                )
            }
            Self::Short => format!("Toggle {period}-cycles"),
        }
    }

    fn draw_external_ray(self, include_orbit: bool, select_landing_point: bool) -> String
    {
        match self {
            Self::Long => {
                if select_landing_point {
                    return "Draw/hide an external ray and select its landing point on active image."
                        .to_owned();
                }
                if include_orbit {
                    return "Draw/hide an external ray, together with its orbit.".to_owned();
                }
                "Draw/hide an external ray on active image.".to_owned()
            }
            Self::Short => {
                if select_landing_point {
                    return "Ray to Point...".to_owned();
                }
                if include_orbit {
                    return "Ray orbit...".to_owned();
                }
                "Draw Ray...".to_owned()
            }
        }
    }

    fn draw_contour(self, contour_type: ContourType) -> String
    {
        match self {
            Self::Long => match contour_type {
                ContourType::Equipotential => "Draw equipotential through selection.".to_owned(),
                ContourType::Multiplier(..) => {
                    "Draw a contour for the multiplier map on dynamical varieties.".to_owned()
                }
                ContourType::ExtendRay => {
                    "Extend an external ray outwards from the selection.".to_owned()
                }
                ContourType::InwardRay => {
                    "Try to draw an external ray inwards from the selection.".to_owned()
                }
            },
            Self::Short => match contour_type {
                ContourType::Equipotential => "Equipotential".to_owned(),
                ContourType::Multiplier(..) => "Multiplier Contour".to_owned(),
                ContourType::ExtendRay => "Extend Ray".to_owned(),
                ContourType::InwardRay => "Inward Ray".to_owned(),
            },
        }
    }

    fn pan(self, x: f64, y: f64) -> String
    {
        if matches!(self, Self::Short) {
            return "Pan View".to_owned();
        }
        if x == 0. {
            return if y > 0. {
                format!("Pan up by {}%", y * 100.)
            } else {
                format!("Pan down by {}%", y * 100.)
            };
        }
        if y == 0. {
            return if x > 0. {
                format!("Pan right by {}%", x * 100.)
            } else {
                format!("Pan left by {}%", -x * 100.)
            };
        }
        format!("Pan the view (x: {x}, y: {y})")
    }

    fn zoom(self, scale: f64) -> String
    {
        match self {
            Self::Long => format!("Zoom {} (scale: {:.2})", in_or_out(scale), scale),
            Self::Short => format!("Zoom {}", in_or_out(scale)),
        }
    }

    fn scale_max_iter(self, scale: f64) -> String
    {
        match self {
            Self::Long => {
                format!(
                    "{} max iterations on active image (factor: {scale})",
                    inc_or_dec(scale)
                )
            }
            Self::Short => format!("{} iters", inc_or_dec(scale)),
        }
    }

    fn scale_palette_period(self, scale: f64) -> String
    {
        match self {
            Self::Long => format!("{} the period of the color palette.", inc_or_dec(scale)),
            Self::Short => format!("{} density", inc_or_dec(1.0 / scale)),
        }
    }

    fn compute_mode(self, change: ChangeBoolean) -> String
    {
        match self {
            Self::Long => match change {
                ChangeBoolean::Enable => {
                    "Use distance estimation to color escape regions".to_owned()
                }
                ChangeBoolean::Disable => "Use Green's function to color escape regions".to_owned(),
                ChangeBoolean::Toggle => {
                    "Cycle between exterior coloring modes (smooth potential and distance estimate)."
                        .to_owned()
                }
            },
            Self::Short => match change {
                ChangeBoolean::Enable => "Distance Estimation".to_owned(),
                ChangeBoolean::Disable => "External Potential".to_owned(),
                ChangeBoolean::Toggle => "Cycle Outcoloring".to_owned(),
            },
        }
    }

    fn set_coloring(self, algorithm: &IncoloringAlgorithm) -> String
    {
        use IncoloringAlgorithm::{
            InternalPotential, Multiplier, Period, PeriodMultiplier, PotentialAndPeriod, Preperiod,
            PreperiodPeriod, Solid,
        };

        match self {
            Self::Long => match algorithm {
                Solid => "Color bounded components black.".to_owned(),
                Period => "Color bounded components by period".to_owned(),
                PeriodMultiplier => {
                    "Color bounded components by period and norm of multiplier".to_owned()
                }
                Multiplier => "Color bounded components by multiplier".to_owned(),
                Preperiod => "Color bounded components by convergence time".to_owned(),
                InternalPotential { .. } => {
                    "Color bounded components by internal potential (Kœnigs or Böttcher map)"
                        .to_owned()
                }
                PreperiodPeriod { .. } => {
                    "Color bounded components by period and convergence time".to_owned()
                }
                PotentialAndPeriod { .. } => {
                    "Color bounded components by period and internal potential".to_owned()
                }
            },
            Self::Short => match algorithm {
                Solid => "Black".to_owned(),
                Period => "Period".to_owned(),
                PeriodMultiplier => "Period + Multiplier".to_owned(),
                Multiplier => "Multiplier".to_owned(),
                Preperiod => "Convergence time".to_owned(),
                InternalPotential { .. } => "Internal Potential".to_owned(),
                PreperiodPeriod { .. } => "Period + Conv. Time".to_owned(),
                PotentialAndPeriod { .. } => "Period + Potential".to_owned(),
            },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChangeBoolean
{
    Enable,
    Disable,
    Toggle,
}
impl ChangeBoolean
{
    pub const fn switch(&self, target: &mut bool)
    {
        match self {
            Self::Enable => {
                *target = true;
            }
            Self::Disable => {
                *target = false;
            }
            Self::Toggle => {
                *target ^= true;
            }
        }
    }
}
impl std::fmt::Display for ChangeBoolean
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Self::Enable => write!(f, "Enable"),
            Self::Disable => write!(f, "Disable"),
            Self::Toggle => write!(f, "Toggle"),
        }
    }
}
