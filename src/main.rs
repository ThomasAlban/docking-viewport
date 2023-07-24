use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use egui_dock::{DockArea, NodeIndex, Style, Tree};

#[derive(Resource)]
struct DockTree(Tree<String>);
impl Default for DockTree {
    fn default() -> Self {
        let mut tree = Tree::new(vec!["tab1".to_owned(), "tab2".to_owned()]);
        // You can modify the tree before constructing the dock
        let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        let [_, _] = tree.split_below(a, 0.7, vec!["tab4".to_owned()]);
        let [_, _] = tree.split_below(b, 0.5, vec!["tab5".to_owned()]);
        Self(tree)
    }
}

struct TabViewer {}
impl egui_dock::TabViewer for TabViewer {
    type Tab = String;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_ui)
        .run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(DockTree::default());
}

fn update_ui(mut contexts: EguiContexts, mut docktree: ResMut<DockTree>) {
    let ctx = contexts.ctx_mut();

    DockArea::new(&mut docktree.as_mut().0)
        .style(Style::from_egui(ctx.style().as_ref()))
        .show(ctx, &mut TabViewer {});
}
