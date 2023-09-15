use bevy::prelude::{Component, Event, Resource, Vec2};
use bevy_ecs_tilemap::tiles::TilePos;

#[derive(Component, Clone, Debug, PartialEq)]
pub enum TileMaterial {
    Wall,
    Floor,
}

#[derive(Resource)]
pub struct RogBrush {
    pub material: TileMaterial,
    pub size: usize,
}

impl RogBrush {
    pub fn default() -> Self {
        Self {
            material: TileMaterial::Wall,
            size: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reversible<T> {
    pub value: T,
    pub undo: T,
}

impl<T> Reversible<T> {
    pub fn new(value: T, undo: T) -> Self {
        Self { value, undo }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditAction {
    SetMaterial(TileMaterial),
    SetSize(usize),
    PlaceTile {
        tile_pos: TilePos,
        material: TileMaterial,
        size: usize,
    },
}

#[derive(Event, Clone, Debug, PartialEq)]
pub struct AddEditActionEvent(pub EditAction);

#[derive(Event, Clone, Debug, PartialEq)]
pub struct UndoEditActionEvent;
#[derive(Event, Clone, Debug, PartialEq)]
pub struct RedoEditActionEvent;

#[derive(Resource)]
pub struct ActionStack {
    cursor: usize,
    stack: Vec<Reversible<EditAction>>,
}

impl ActionStack {
    pub fn default() -> Self {
        Self {
            cursor: 0,
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, value: EditAction, undo: EditAction) {
        if self.cursor < self.stack.len() {
            self.stack.truncate(self.cursor);
        }

        self.stack.push(Reversible { value, undo });
        self.cursor = self.stack.len();
    }

    pub fn undo(&mut self) -> Option<EditAction> {
        if self.cursor > 0 {
            self.cursor -= 1;
            Some(self.stack[self.cursor].undo.clone())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<EditAction> {
        if self.cursor < self.stack.len() {
            self.cursor += 1;
            Some(self.stack[self.cursor - 1].value.clone())
        } else {
            None
        }
    }
}
