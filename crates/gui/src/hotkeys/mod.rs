pub mod keyboard_shortcuts;
use crate::{
    actions::{Action, ChangeBoolean},
    marked_points::ContourType,
    pane::id::{PaneID::*, PaneSelection::*},
};
use dynamo_color::IncoloringAlgorithm;
use keyboard_shortcuts::*;
use seq_macro::seq;

use egui::{KeyboardShortcut, ModifierNames, RichText};

pub struct Hotkey
{
    pub(super) shortcut: Option<KeyboardShortcut>,
    pub(super) action: Action,
    pub(super) show_in_menu: bool,
    /// Custom action to perform instead of standard one if called from menu
    /// Defaults to `action` if this is set to `None`.
    pub(super) menu_action_override: Option<Action>,
}
impl Hotkey
{
    #[must_use]
    pub const fn action(&self) -> &Action
    {
        &self.action
    }
    #[must_use]
    pub fn menu_action(&self) -> Option<&Action>
    {
        if self.show_in_menu {
            self.menu_action_override.as_ref().or(Some(&self.action))
        } else {
            None
        }
    }
    #[must_use]
    pub fn shortcut_text(&self) -> Option<RichText>
    {
        Some(
            RichText::new(self.shortcut?.format(&ModifierNames::NAMES, true))
                .strong()
                .color(epaint::Color32::LIGHT_GRAY),
        )
    }
}

use Action::*;

pub static FILE_HOTKEYS: [Hotkey; 6] = [
    Hotkey {
        shortcut: Some(CTRL_Q),
        action: Quit,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_W),
        action: Close,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_T),
        action: NewTab,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_S),
        action: SaveImage(ActivePane),
        show_in_menu: false,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: None,
        action: SaveImage(Id(Parent)),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: None,
        action: SaveImage(Id(Child)),
        show_in_menu: true,
        menu_action_override: None,
    },
];

pub static PALETTE_HOTKEYS: [Hotkey; 9] = [
    Hotkey {
        shortcut: Some(CTRL_K),
        action: SavePalette(ActivePane),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_L),
        action: LoadPalette(BothPanes),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_B),
        action: SetPaletteBlack,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_W),
        action: SetPaletteWhite,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_R),
        action: RandomizePalette,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_UP),
        action: ScalePalettePeriod(1.25),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_DOWN),
        action: ScalePalettePeriod(0.8),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_LEFT),
        action: ShiftPalettePhase(-0.02),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_RIGHT),
        action: ShiftPalettePhase(0.02),
        show_in_menu: true,
        menu_action_override: None,
    },
];

seq!(n in 1..=6 {
pub static CYCLES_HOTKEYS: [Hotkey; 12] = [
    #(
        Hotkey {
            shortcut: Some(CTRL_~n),
            action: ToggleCycles(Id(Child), n),
            show_in_menu: true,
            menu_action_override: None,
        },
        Hotkey {
            shortcut: Some(CTRL_SHIFT_~n),
            action: ToggleCycles(Id(Parent), n),
            show_in_menu: false,
            menu_action_override: None,
        },
    )*
];
});

pub static ANNOTATION_HOTKEYS: [Hotkey; 15] = [
    // External ray
    Hotkey {
        shortcut: Some(KEY_E),
        action: DrawExternalRay {
            include_orbit: false,
            select_landing_point: false,
        },
        show_in_menu: true,
        menu_action_override: None,
    },
    // External ray to point
    Hotkey {
        shortcut: Some(KEY_Y),
        action: DrawExternalRay {
            include_orbit: false,
            select_landing_point: true,
        },
        show_in_menu: true,
        menu_action_override: None,
    },
    // External ray to point
    Hotkey {
        shortcut: Some(CTRL_X),
        action: DrawExternalRay {
            include_orbit: false,
            select_landing_point: true,
        },
        show_in_menu: false,
        menu_action_override: None,
    },
    // Ray orbit
    Hotkey {
        shortcut: Some(SHIFT_O),
        action: DrawExternalRay {
            include_orbit: true,
            select_landing_point: false,
        },
        show_in_menu: true,
        menu_action_override: None,
    },
    // Rays of exact period
    Hotkey {
        shortcut: Some(CTRL_E),
        action: DrawRaysOfPeriod,
        show_in_menu: true,
        menu_action_override: None,
    },
    // Equipotential
    Hotkey {
        shortcut: Some(KEY_G),
        action: DrawContour(ContourType::Equipotential),
        show_in_menu: true,
        menu_action_override: None,
    },
    // Multiplier contour
    Hotkey {
        shortcut: Some(KEY_M),
        action: DrawContour(ContourType::multiplier_auto()),
        show_in_menu: true,
        menu_action_override: None,
    },
    // Many multiplier contours
    Hotkey {
        shortcut: Some(SHIFT_M),
        action: DrawAuxContours,
        show_in_menu: true,
        menu_action_override: None,
    },
    // Extend Ray
    Hotkey {
        shortcut: Some(SHIFT_E),
        action: DrawContour(ContourType::ExtendRay),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_P),
        action: ToggleCritical,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(SHIFT_P),
        action: ToggleMarked(ActivePane),
        show_in_menu: false,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_O),
        action: DrawOrbit,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_ESC),
        action: StopFollowing,
        show_in_menu: false,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_C),
        action: ClearOrbit,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(SHIFT_C),
        action: ClearCurves,
        show_in_menu: true,
        menu_action_override: None,
    },
];

pub static SELECTION_HOTKEYS: [Hotkey; 5] = [
    Hotkey {
        shortcut: Some(KEY_I),
        action: ToggleSelectionMarker,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_INSERT),
        action: EnterCoordinates,
        show_in_menu: true,
        menu_action_override: None,
    },
    // Apply map on dynamical plane
    Hotkey {
        shortcut: Some(KEY_F),
        action: MapSelection,
        show_in_menu: true,
        menu_action_override: None,
    },
    // Find nearby periodic point
    Hotkey {
        shortcut: Some(CTRL_F),
        action: FindPeriodicPoint,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(SHIFT_SPACE),
        action: ResetSelection,
        show_in_menu: true,
        menu_action_override: None,
    },
];

pub static IMAGE_HOTKEYS: [Hotkey; 14] = [
    // Hotkey {
    //     shortcut: Some(KEY_H),
    //     action: PromptImageHeight,
    //     show_in_menu: true,
    //     menu_action_override: None,
    // },
    Hotkey {
        shortcut: Some(KEY_L),
        action: ToggleLiveMode,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_EQUALS),
        action: ScaleMaxIter(2.0),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_MINUS),
        action: ScaleMaxIter(0.5),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(SHIFT_LEFT),
        action: Pan(-0.01, 0.),
        show_in_menu: false,
        menu_action_override: Some(Pan(-0.1, 0.)),
    },
    Hotkey {
        shortcut: Some(SHIFT_RIGHT),
        action: Pan(0.01, 0.),
        show_in_menu: false,
        menu_action_override: Some(Pan(0.1, 0.)),
    },
    Hotkey {
        shortcut: Some(SHIFT_UP),
        action: Pan(0., 0.01),
        show_in_menu: false,
        menu_action_override: Some(Pan(0., 0.1)),
    },
    Hotkey {
        shortcut: Some(SHIFT_DOWN),
        action: Pan(0., -0.01),
        show_in_menu: false,
        menu_action_override: Some(Pan(0., -0.1)),
    },
    Hotkey {
        shortcut: Some(KEY_Z),
        action: Zoom(0.8),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_Z),
        action: Zoom(0.125),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_V),
        action: Zoom(1.25),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_V),
        action: Zoom(8.),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_SPACE),
        action: CenterOnSelection,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(CTRL_P),
        action: CycleActivePlane,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_HOME),
        action: ResetView,
        show_in_menu: true,
        menu_action_override: None,
    },
];

pub static INCOLORING_HOTKEYS: [Hotkey; 8] = [
    Hotkey {
        shortcut: Some(KEY_0),
        action: SetColoring(IncoloringAlgorithm::Solid),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_1),
        action: SetColoring(IncoloringAlgorithm::Period),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_2),
        action: SetColoring(IncoloringAlgorithm::PeriodMultiplier),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_3),
        action: SetColoring(IncoloringAlgorithm::Multiplier),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_4),
        action: SetColoring(IncoloringAlgorithm::Preperiod),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_5),
        action: SetColoringInternalPotential,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_6),
        action: SetColoringPreperiodPeriod,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_7),
        action: SetColoringPotentialPeriod,
        show_in_menu: true,
        menu_action_override: None,
    },
];

pub static OUTCOLORING_HOTKEYS: [Hotkey; 4] = [
    Hotkey {
        shortcut: Some(KEY_J),
        action: ToggleEscapePhaseColoring,
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: Some(KEY_D),
        action: CycleComputeMode(ActivePane, ChangeBoolean::Toggle),
        show_in_menu: false,
        menu_action_override: Some(CycleComputeMode(ActivePane, ChangeBoolean::Enable)),
    },
    Hotkey {
        shortcut: None,
        action: CycleComputeMode(BothPanes, ChangeBoolean::Disable),
        show_in_menu: true,
        menu_action_override: None,
    },
    Hotkey {
        shortcut: None,
        action: CycleComputeMode(BothPanes, ChangeBoolean::Enable),
        show_in_menu: true,
        menu_action_override: None,
    },
];
