use dynamo_gui::interface::Interface;
use egui::{Color32, Ui};

pub enum Action
{
    ChangeFractal(Box<dyn Interface>),
}

#[derive(Default)]
pub enum Item
{
    ChangeFractal(fn() -> Box<dyn Interface>),
    Submenu(Box<dyn Fn() -> State>),
    #[default]
    GoToParent,
}

pub struct Tile
{
    name: String,
    item: Item,
}

impl Tile
{
    fn draw_rect(&self, ui: &mut Ui) -> egui::Response
    {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(220.0, 40.0), egui::Sense::click());

        let (color, text_color) = if response.is_pointer_button_down_on() {
            (self.flash_color(), Color32::BLACK)
        } else {
            (self.color(), Color32::WHITE)
        };

        ui.painter().rect_filled(rect, 6.0, color);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.name,
            egui::FontId::default(),
            text_color,
        );
        response
    }

    const fn color(&self) -> Color32
    {
        match self.item {
            Item::ChangeFractal(_) => Color32::from_rgb(43, 37, 121),
            Item::Submenu(_) => Color32::from_rgb(44, 96, 60),
            Item::GoToParent => Color32::from_rgb(70, 70, 70),
        }
    }

    const fn flash_color(&self) -> Color32
    {
        match self.item {
            Item::ChangeFractal(_) => Color32::from_rgb(144, 144, 237),
            Item::Submenu(_) => Color32::from_rgb(60, 179, 113),
            Item::GoToParent => Color32::from_rgb(169, 169, 169),
        }
    }
}

#[derive(Default)]
pub struct State
{
    pub tiles: Vec<Tile>,
}

impl State
{
    #[must_use]
    pub fn submenu() -> Self
    {
        Self::default().with_tile("Go Back", Item::GoToParent)
    }

    #[must_use]
    pub fn with_submenu<F>(self, name: &str, make_menu: F) -> Self
    where
        F: Fn() -> Self + 'static,
    {
        let item = Item::Submenu(Box::new(make_menu));
        self.with_tile(name, item)
    }

    #[must_use]
    pub fn with_fractal_button(
        self,
        name: &str,
        create_interface: fn() -> Box<dyn Interface>,
    ) -> Self
    {
        let item = Item::ChangeFractal(create_interface);
        self.with_tile(name, item)
    }

    pub fn add_submenu<F>(&mut self, name: &str, make_menu: F)
    where
        F: Fn() -> Self + 'static,
    {
        let item = Item::Submenu(Box::new(make_menu));
        self.add_tile(name, item);
    }

    pub fn add_fractal_button(&mut self, name: &str, create_interface: fn() -> Box<dyn Interface>)
    {
        let item = Item::ChangeFractal(create_interface);
        self.add_tile(name, item);
    }

    fn with_tile(mut self, name: &str, item: Item) -> Self
    {
        let tile = Tile {
            name: name.to_owned(),
            item,
        };
        self.tiles.push(tile);
        self
    }

    fn add_tile(&mut self, name: &str, item: Item)
    {
        let tile = Tile {
            name: name.to_owned(),
            item,
        };
        self.tiles.push(tile);
    }
}

#[derive(Default)]
enum NavAction
{
    #[default]
    DoNothing,
    Ascend,
    Descend(State),
}

#[derive(Default)]
pub struct Menu
{
    pub state: State,
    above: Vec<State>,
}

impl Menu
{
    #[must_use]
    pub const fn new(state: State) -> Self
    {
        Self {
            state,
            above: Vec::new(),
        }
    }

    pub fn show_and_get_action(&mut self, ui: &mut Ui) -> Option<Action>
    {
        let mut nav_action: NavAction = NavAction::DoNothing;

        ui.add_space(50.);
        for tile in &mut self.state.tiles {
            if tile.draw_rect(ui).clicked() {
                match &tile.item {
                    Item::GoToParent => {
                        nav_action = NavAction::Ascend;
                        break;
                    }
                    Item::ChangeFractal(cons) => return Some(Action::ChangeFractal(cons())),
                    Item::Submenu(create_menu) => {
                        nav_action = NavAction::Descend(create_menu());
                        break;
                    }
                }
            }
        }
        match nav_action {
            NavAction::Ascend => {
                self.state = self.above.pop()?;
            }
            NavAction::Descend(state) => {
                let old_state = std::mem::replace(&mut self.state, state);
                self.above.push(old_state);
            }
            NavAction::DoNothing => {}
        }
        None
    }
}
