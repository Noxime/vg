use std::{cell::Cell, sync::Arc};

use egui::*;
use egui_tiles::{
    Behavior, Container, SimplificationOptions, Tabs, Tile, TileId, Tiles, Tree, UiResponse,
};
use egui_winit::winit::{event::Event as WinitEvent, event_loop::EventLoopWindowTarget};

mod controller;
mod logger;

use crate::tracing::Tracing;

use self::{controller::Controller, logger::Logger};
pub struct EditorUi {
    tree: Tree<Pane>,
    behavior: TreeBehavior,
    tracing: Arc<Tracing>,
}

struct Pane {
    name: Cell<String>,
    kind: PaneKind,
}

enum PaneKind {
    Logger(Logger),
    Controller(Controller),
}

enum NewPane {
    Logger,
    Controller,
}

struct TreeBehavior {
    insert: Option<(TileId, NewPane)>,
    remove: Option<TileId>,
}

impl TreeBehavior {
    fn new() -> TreeBehavior {
        TreeBehavior {
            insert: None,
            remove: None,
        }
    }
}

impl Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        let s = pane.name.take();
        pane.name.set(s.clone());
        s.into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, _: TileId, pane: &mut Pane) -> UiResponse {
        match &mut pane.kind {
            PaneKind::Logger(logger) => logger.ui(ui),
            PaneKind::Controller(controller) => controller.ui(ui),
        };

        UiResponse::None
    }

    fn top_bar_right_ui(
        &mut self,
        tiles: &Tiles<Pane>,
        ui: &mut Ui,
        id: TileId,
        tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        ui.menu_button("New", |ui| {
            if ui.button("Logger").clicked() {
                self.insert = Some((id, NewPane::Logger));
            }
            if ui.button("Controller").clicked() {
                self.insert = Some((id, NewPane::Controller));
            }
        });

        let Some(id) = tabs.active else { return };
        let Some(active) = tiles.get(id) else { return };
        let Tile::Pane(pane) = active else { return };
        let mut name = pane.name.take();

        ui.menu_button("Rename", |ui| {
            ui.add(TextEdit::singleline(&mut name)).request_focus();
        });

        if ui.button("Close").clicked() {
            self.remove = Some(id);
        }

        pane.name.set(name);
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            prune_empty_tabs: true,
            prune_empty_containers: true,
            prune_single_child_tabs: true,
            prune_single_child_containers: true,
            all_panes_must_have_tabs: true,
            join_nested_linear_containers: true,
        }
    }
}

impl EditorUi {
    pub fn new(tracing: Arc<Tracing>) -> EditorUi {
        let mut tiles = Tiles::default();

        let root = tiles.insert_tab_tile(vec![]);

        let mut this = EditorUi {
            tree: Tree::new("tree", root, tiles),
            behavior: TreeBehavior::new(),
            tracing,
        };

        let logger = this.tree.tiles.insert_pane(this.create(NewPane::Logger));
        let engine = this
            .tree
            .tiles
            .insert_pane(this.create(NewPane::Controller));
        let root = this.tree.tiles.insert_vertical_tile(vec![engine, logger]);

        this.set_root(root);
        this
    }

    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.tree.is_empty() {
                self.behavior.insert = Some((self.root(), NewPane::Controller));
            }

            // Handle new UI elements n shit
            if let Some((id, kind)) = self.behavior.insert.take() {
                self.create_child(id, kind);
            }

            if let Some(id) = self.behavior.remove.take() {
                self.tree.tiles.remove(id);
            }

            self.tree.ui(&mut self.behavior, ui);
        });
    }

    pub fn event(&mut self, event: &WinitEvent<()>, target: &EventLoopWindowTarget<()>) {
        for tile in self.tree.tiles.tiles_mut() {
            if let Tile::Pane(pane) = tile {
                match &mut pane.kind {
                    PaneKind::Logger(_) => (),
                    PaneKind::Controller(c) => c.event(event, target),
                }
            }
        }
    }

    /// Get the ID of the root tile, creating if it does not exist somehow
    fn root(&mut self) -> TileId {
        if let Some(id) = self.tree.root() {
            return id;
        }

        let root = self.tree.tiles.insert_tab_tile(vec![]);
        self.set_root(root)
    }

    fn set_root(&mut self, id: TileId) -> TileId {
        self.tree = Tree::new("tree", id, std::mem::take(&mut self.tree.tiles));
        self.tree.root().unwrap()
    }

    /// Create a new pane from definition
    fn create(&self, kind: NewPane) -> Pane {
        match kind {
            NewPane::Logger => Pane {
                name: Cell::new("Logger".into()),
                kind: PaneKind::Logger(Logger::new(Arc::clone(&self.tracing))),
            },
            NewPane::Controller => Pane {
                name: Cell::new("Controller".into()),
                kind: PaneKind::Controller(Controller::new()),
            },
        }
    }

    /// Create a new child for tile
    fn create_child(&mut self, id: TileId, kind: NewPane) {
        let child = self.tree.tiles.insert_pane(self.create(kind));
        self.ensure_tabs(id).add_child(child);
    }

    /// Ensure this **pane** is wrapped in tabs
    fn ensure_tabs(&mut self, id: TileId) -> &mut Tabs {
        match self.tree.tiles.get(id) {
            Some(Tile::Pane(_)) => {
                let tabs = Tile::Container(Container::Tabs(Tabs::new(vec![id])));
                self.tree.tiles.insert(id, tabs);
            }
            _ => {}
        }

        if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(id) {
            return tabs;
        }

        panic!()
    }
}
