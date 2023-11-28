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
    pub(super) bonus_action: Option<Action>,
    pub(super) show_in_menu: bool,
    /// Custom action to perform instead of standard one if called from menu
    /// Defaults to `action` if this is set to `None`.
    pub(super) menu_action_override: Option<Action>,
}
impl Hotkey
{
    #[must_use]
    pub const fn new(action: Action) -> Self
    {
        Self {
            shortcut: None,
            action,
            bonus_action: None,
            show_in_menu: true,
            menu_action_override: None,
        }
    }

    #[must_use]
    pub const fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self
    {
        self.shortcut = Some(shortcut);
        self
    }
    #[must_use]
    pub const fn action(mut self, action: Action) -> Self
    {
        self.action = action;
        self
    }
    #[must_use]
    pub const fn bonus_action(mut self, action: Action) -> Self
    {
        self.bonus_action = Some(action);
        self
    }
    #[must_use]
    pub const fn menu_action_override(mut self, action: Action) -> Self
    {
        self.menu_action_override = Some(action);
        self
    }
    #[must_use]
    pub const fn hide_in_menu(mut self) -> Self
    {
        self.show_in_menu = false;
        self
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

pub const FILE_HOTKEYS: [Hotkey; 6] = [
    Hotkey::new(Quit).shortcut(CTRL_Q),
    Hotkey::new(Close).shortcut(CTRL_W),
    Hotkey::new(NewTab).shortcut(CTRL_T),
    Hotkey::new(SaveImage(ActivePane))
        .shortcut(CTRL_S)
        .hide_in_menu(),
    Hotkey::new(SaveImage(Id(Parent))),
    Hotkey::new(SaveImage(Id(Child))),
];

pub const PALETTE_HOTKEYS: [Hotkey; 9] = [
    Hotkey::new(SavePalette(ActivePane)).shortcut(CTRL_K),
    Hotkey::new(LoadPalette(BothPanes)).shortcut(CTRL_L),
    Hotkey::new(SetPaletteBlack).shortcut(KEY_B),
    Hotkey::new(SetPaletteWhite).shortcut(KEY_W),
    Hotkey::new(RandomizePalette).shortcut(KEY_R),
    Hotkey::new(ScalePalettePeriod(1.25)).shortcut(KEY_UP),
    Hotkey::new(ScalePalettePeriod(0.8)).shortcut(KEY_DOWN),
    Hotkey::new(ShiftPalettePhase(-0.02)).shortcut(KEY_LEFT),
    Hotkey::new(ShiftPalettePhase(0.02)).shortcut(KEY_RIGHT),
];

seq!(n in 1..=6 {
pub const CYCLES_HOTKEYS: [Hotkey; 12] = [
    #(
        Hotkey::new(ToggleCycles(Id(Child), n)).shortcut(CTRL_~n),
        Hotkey::new(ToggleCycles(Id(Parent), n)).shortcut(CTRL_SHIFT_~n).hide_in_menu(),
    )*
];
});

pub const ANNOTATION_HOTKEYS: [Hotkey; 17] = [
    // External ray
    Hotkey::new(DrawExternalRay {
        include_orbit: false,
        select_landing_point: false,
    })
    .shortcut(KEY_E),
    // External ray to point
    Hotkey::new(DrawExternalRay {
        include_orbit: false,
        select_landing_point: true,
    })
    .shortcut(KEY_Y),
    // External ray to point
    Hotkey::new(DrawExternalRay {
        include_orbit: false,
        select_landing_point: true,
    })
    .shortcut(CTRL_X)
    .hide_in_menu(),
    // Ray orbit
    Hotkey::new(DrawExternalRay {
        include_orbit: true,
        select_landing_point: false,
    })
    .shortcut(SHIFT_O),
    // Rays of exact period
    Hotkey::new(DrawRaysOfPeriod).shortcut(CTRL_E),
    // Equipotential
    Hotkey::new(DrawContour(ContourType::Equipotential)).shortcut(KEY_G),
    // Multiplier contour
    Hotkey::new(DrawContour(ContourType::multiplier_auto())).shortcut(KEY_M),
    // Many multiplier contours
    Hotkey::new(DrawAuxContours).shortcut(SHIFT_M),
    // Extend Ray
    Hotkey::new(DrawContour(ContourType::ExtendRay)).shortcut(SHIFT_E),
    // Inward Ray
    Hotkey::new(DrawContour(ContourType::InwardRay)).shortcut(SHIFT_R),
    // Bidirectional Ray
    Hotkey::new(DrawContour(ContourType::ExtendRay))
        .bonus_action(DrawContour(ContourType::InwardRay))
        .shortcut(SHIFT_T)
        .hide_in_menu(),
    Hotkey::new(ToggleCritical).shortcut(KEY_P),
    Hotkey::new(ToggleMarked(ActivePane))
        .shortcut(SHIFT_P)
        .hide_in_menu(),
    Hotkey::new(DrawOrbit).shortcut(KEY_O),
    Hotkey::new(StopFollowing).shortcut(KEY_ESC).hide_in_menu(),
    Hotkey::new(ClearOrbit).shortcut(KEY_C),
    Hotkey::new(ClearCurves).shortcut(SHIFT_C),
];

pub const SELECTION_HOTKEYS: [Hotkey; 5] = [
    Hotkey::new(ToggleSelectionMarker).shortcut(KEY_I),
    Hotkey::new(EnterCoordinates).shortcut(KEY_INSERT),
    // Apply map on dynamical plane
    Hotkey::new(MapSelection).shortcut(KEY_F),
    // Find nearby periodic point
    Hotkey::new(FindPeriodicPoint).shortcut(CTRL_F),
    Hotkey::new(ResetSelection).shortcut(SHIFT_SPACE),
];

pub const IMAGE_HOTKEYS: [Hotkey; 14] = [
    // Hotkey {
    //     shortcut: Some(KEY_H),
    //     action: PromptImageHeight,
    //     show_in_menu: true,
    //     menu_action_override: None,
    // },
    Hotkey::new(ToggleLiveMode).shortcut(KEY_L),
    Hotkey::new(ScaleMaxIter(2.0)).shortcut(KEY_EQUALS),
    Hotkey::new(ScaleMaxIter(0.5)).shortcut(KEY_MINUS),
    Hotkey::new(Pan(-0.01, 0.))
        .shortcut(SHIFT_LEFT)
        .hide_in_menu()
        .menu_action_override(Pan(-0.1, 0.)),
    Hotkey::new(Pan(0.01, 0.))
        .shortcut(SHIFT_RIGHT)
        .hide_in_menu()
        .menu_action_override(Pan(0.1, 0.)),
    Hotkey::new(Pan(0., 0.01))
        .shortcut(SHIFT_UP)
        .hide_in_menu()
        .menu_action_override(Pan(0., 0.1)),
    Hotkey::new(Pan(0., -0.01))
        .shortcut(SHIFT_DOWN)
        .hide_in_menu()
        .menu_action_override(Pan(0., -0.1)),
    Hotkey::new(Zoom(0.8)).shortcut(KEY_Z),
    Hotkey::new(Zoom(0.125)).shortcut(CTRL_Z),
    Hotkey::new(Zoom(1.25)).shortcut(KEY_V),
    Hotkey::new(Zoom(8.)).shortcut(CTRL_V),
    Hotkey::new(CenterOnSelection).shortcut(KEY_SPACE),
    Hotkey::new(CycleActivePlane).shortcut(CTRL_P),
    Hotkey::new(ResetView).shortcut(KEY_HOME),
];

pub const INCOLORING_HOTKEYS: [Hotkey; 8] = [
    Hotkey::new(SetColoring(IncoloringAlgorithm::Solid)).shortcut(KEY_0),
    Hotkey::new(SetColoring(IncoloringAlgorithm::Period)).shortcut(KEY_1),
    Hotkey::new(SetColoring(IncoloringAlgorithm::PeriodMultiplier)).shortcut(KEY_2),
    Hotkey::new(SetColoring(IncoloringAlgorithm::Multiplier)).shortcut(KEY_3),
    Hotkey::new(SetColoring(IncoloringAlgorithm::Preperiod)).shortcut(KEY_4),
    Hotkey::new(SetColoringInternalPotential).shortcut(KEY_5),
    Hotkey::new(SetColoringPreperiodPeriod).shortcut(KEY_6),
    Hotkey::new(SetColoringPotentialPeriod).shortcut(KEY_7),
];

pub const OUTCOLORING_HOTKEYS: [Hotkey; 4] = [
    Hotkey::new(ToggleEscapePhaseColoring).shortcut(KEY_J),
    Hotkey::new(CycleComputeMode(ActivePane, ChangeBoolean::Toggle))
        .shortcut(KEY_D)
        .hide_in_menu()
        .menu_action_override(CycleComputeMode(ActivePane, ChangeBoolean::Enable)),
    Hotkey::new(CycleComputeMode(BothPanes, ChangeBoolean::Disable)),
    Hotkey::new(CycleComputeMode(BothPanes, ChangeBoolean::Enable)),
];
