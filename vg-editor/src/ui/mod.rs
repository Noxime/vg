use std::sync::Arc;

use egui::*;
use egui_tiles::{
    Behavior, Container, SimplificationOptions, Tabs, Tile, TileId, Tiles, Tree, UiResponse,
};

mod logger;

use crate::tracing::Tracing;

use self::logger::Logger;
pub struct EditorUi {
    tree: Tree<Pane>,
    behavior: TreeBehavior,
    tracing: Arc<Tracing>,
}

struct Pane {
    name: String,
    kind: PaneKind,
}

enum PaneKind {
    Logger(Logger),
}

enum NewPane {
    Logger,
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
        pane.name.to_string().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        ui.horizontal(|ui| {
            ui.menu_button("Edit", |ui| {
                TextEdit::singleline(&mut pane.name)
                    .desired_width(60.0)
                    .show(ui);
            });

            if ui.small_button("Close tab").clicked() {
                self.remove = Some(tile_id);
            }
        });

        match &mut pane.kind {
            PaneKind::Logger(logger) => logger.ui(ui),
        };

        UiResponse::None
    }

    fn top_bar_rtl_ui(
        &mut self,
        _tiles: &Tiles<Pane>,
        ui: &mut Ui,
        id: TileId,
        _tabs: &egui_tiles::Tabs,
    ) {
        if ui.button("Close all tabs").clicked() {
            self.remove = Some(id);
        }

        ui.menu_button("Open", |ui| {
            if ui.button("Logger").clicked() {
                self.insert = Some((id, NewPane::Logger));
            }
        });
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            prune_empty_tabs: true,
            prune_empty_containers: true,
            prune_single_child_tabs: true,
            prune_single_child_containers: true,
            all_panes_must_have_tabs: true,
            join_nested_linear_containerss: true,
        }
    }
}

impl EditorUi {
    pub fn new(tracing: Arc<Tracing>) -> EditorUi {
        let mut tiles = Tiles::default();

        let root = tiles.insert_tab_tile(vec![]);

        EditorUi {
            tree: Tree::new(root, tiles),
            behavior: TreeBehavior::new(),
            tracing,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.tree.is_empty() {
                if ui.button("uh oh").clicked() {
                    self.behavior.insert = Some((self.root(), NewPane::Logger));
                }
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

    /// Get the ID of the root tile, creating if it does not exist somehow
    fn root(&mut self) -> TileId {
        if let Some(id) = self.tree.root() {
            return id;
        }

        let root = self.tree.tiles.insert_tab_tile(vec![]);
        self.set_root(root)
    }

    fn set_root(&mut self, id: TileId) -> TileId {
        self.tree = Tree::new(id, std::mem::take(&mut self.tree.tiles));
        self.tree.root().unwrap()
    }

    /// Create a new pane from definition
    fn create(&self, kind: NewPane) -> Pane {
        match kind {
            NewPane::Logger => Pane {
                name: "Logger".into(),
                kind: PaneKind::Logger(Logger::new(Arc::clone(&self.tracing))),
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
