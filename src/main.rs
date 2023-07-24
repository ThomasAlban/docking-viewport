use bevy::{
    math::vec2,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::PrimaryWindow,
    winit::{UpdateMode, WinitSettings},
};
use bevy_egui::{
    egui::{self, TextureId},
    EguiContexts, EguiPlugin, EguiUserTextures,
};
use egui_dock::{DockArea, NodeIndex, Style, Tree};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Example Dockspace With Viewport".into(),
                ..default()
            }),
            ..default()
        }))
        // don't update the window while it is unfocussed to save on performance
        // remove this if you want it always to update no matter what
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::ReactiveLowPower {
                max_wait: std::time::Duration::MAX,
            },
            ..default()
        })
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup_docktree)
        .add_systems(Startup, setup_viewport)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_ui)
        .add_systems(Update, rotate_cube)
        .run();
}

// stores the docktree containing all the tabs
#[derive(Deref, DerefMut, Resource)]
struct DockTree(Tree<String>);

// stores the image which the camera renders to, so that we can display a viewport inside a tab
#[derive(Deref, Resource)]
struct Viewport(Handle<Image>);

// marker struct for the example cube
#[derive(Component)]
struct ExampleCube;

// this tells egui how to render each tab
struct TabViewer<'a> {
    // add into here any data that needs to be passed into any tabs
    viewport_image: &'a mut Image,
    viewport_tex_id: TextureId,
    window_scale_factor: f64,
    // for example, we pass in the cube_material from the update_ui system so it can be edited in this UI
    cube_material: &'a mut StandardMaterial,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    // each tab will be distinguished by a string - its name
    type Tab = String;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // we can do different things inside the tab depending on its name
        match tab.as_str() {
            "Viewport" => {
                let viewport_size = vec2(ui.available_width(), ui.available_height());
                // resize the viewport if needed
                if self.viewport_image.size().as_uvec2() != viewport_size.as_uvec2() {
                    let size = Extent3d {
                        width: viewport_size.x as u32 * self.window_scale_factor as u32,
                        height: viewport_size.y as u32 * self.window_scale_factor as u32,
                        ..default()
                    };
                    self.viewport_image.resize(size);
                }
                // show the viewport image
                ui.image(self.viewport_tex_id, viewport_size.to_array());
                dbg!(viewport_size, self.viewport_image.size());
            }
            "Scene Control" => {
                let mut color = self.cube_material.base_color.as_rgba_f32();
                ui.horizontal(|ui| {
                    ui.label("Edit Cube Color:");
                    ui.color_edit_button_rgba_unmultiplied(&mut color);
                });
                self.cube_material.base_color = color.into();
            }
            // any other tab will just show this basic default UI
            _ => {
                ui.label(format!("Content of {tab}"));
            }
        };
    }
    // show the title of the tab - the 'Tab' type already stores its title anyway
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

fn setup_docktree(mut commands: Commands) {
    // create the docktree
    let mut tree = Tree::new(vec!["Viewport".to_owned(), "Tab 1".to_owned()]);
    // you can modify the tree before constructing the dock
    let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec!["Scene Control".to_owned()]);
    let [_, _] = tree.split_below(a, 0.7, vec!["Tab 2".to_owned()]);
    let [_, _] = tree.split_below(b, 0.5, vec!["Tab 3".to_owned()]);
    let docktree = DockTree(tree);

    commands.insert_resource(docktree);
}

fn setup_viewport(
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // default size (will be immediately overwritten)
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // this is the texture that will be rendered to
    let mut image: Image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    // create a handle to the image
    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone());
    commands.insert_resource(Viewport(image_handle.clone()));

    // spawn a camera which renders to the image handle
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d::default(),
        camera: Camera {
            // render to the image
            target: RenderTarget::Image(image_handle),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(20., 20., 20.))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube mesh and material
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 4.0 }));
    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });
    // example cube
    commands
        .spawn(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(ExampleCube);
    // directional light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(10., 30., 15.))
            .looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    });
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });
}

fn update_ui(
    mut contexts: EguiContexts,
    mut docktree: ResMut<DockTree>,
    viewport: Res<Viewport>,
    mut image_assets: ResMut<Assets<Image>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    material_handle: Query<&mut Handle<StandardMaterial>, With<ExampleCube>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let viewport_image = image_assets
        .get_mut(&viewport)
        .expect("Could not get viewport image");
    let viewport_tex_id = contexts
        .image_id(&viewport)
        .expect("Could not get viewport texture ID");
    let window_scale_factor = window.get_single().unwrap().scale_factor();
    let ctx = contexts.ctx_mut();

    // as an example we get the cube material so it can be edited in the UI
    let cube_material = material_assets
        .get_mut(material_handle.get_single().unwrap())
        .unwrap();

    // menu bar along the top of the screen
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Window", |ui| {
                // toggle each tab on or off
                for tab in &["Viewport", "Scene Control", "Tab 1", "Tab 2", "Tab 3"] {
                    // search for the tab and see if it currently exists
                    let tab_in_docktree = docktree.find_tab(&tab.to_string());
                    if ui
                        .selectable_label(tab_in_docktree.is_some(), *tab)
                        .clicked()
                    {
                        // remove if it exists, else create it
                        if let Some(index) = tab_in_docktree {
                            docktree.remove_tab(index);
                        } else {
                            docktree.push_to_focused_leaf(tab.to_string());
                        }
                    }
                }
            });
        });
    });

    // show the actual dock area
    DockArea::new(&mut docktree)
        .style(Style::from_egui(ctx.style().as_ref()))
        .show(
            ctx,
            &mut TabViewer {
                viewport_image,
                viewport_tex_id,
                window_scale_factor,
                cube_material,
            },
        );
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<ExampleCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}
