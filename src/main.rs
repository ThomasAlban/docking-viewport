use bevy::{
    math::vec2,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
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
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    // each tab will be distinguished by a string - its name
    type Tab = String;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // we can do different things inside the tab depending on its name
        match tab.as_str() {
            "Viewport" => {
                let viewport_size = vec2(ui.available_width(), ui.available_height());
                let viewport_size_array: [f32; 2] = viewport_size.into();

                // resize the viewport if needed
                if self.viewport_image.size() != viewport_size
                // && viewport_size.x != 0.
                // && viewport_size.y != 0.
                {
                    let size = Extent3d {
                        width: viewport_size[0] as u32,
                        height: viewport_size[1] as u32,
                        ..default()
                    };
                    self.viewport_image.resize(size)
                }
                // show the viewport image
                ui.image(self.viewport_tex_id, viewport_size_array);
            }
            _ => {
                ui.label(format!("Content of {tab}"));
            }
        };
    }
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}

fn setup_docktree(mut commands: Commands) {
    // create the docktree
    let mut tree = Tree::new(vec!["Viewport".to_owned(), "tab2".to_owned()]);
    // You can modify the tree before constructing the dock
    let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
    let [_, _] = tree.split_below(a, 0.7, vec!["tab4".to_owned()]);
    let [_, _] = tree.split_below(b, 0.5, vec!["tab5".to_owned()]);
    let docktree = DockTree(tree);

    commands.insert_resource(docktree);
}

fn setup_viewport(
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
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
    mut assets: ResMut<Assets<Image>>,
) {
    let viewport_image = assets
        .get_mut(&viewport)
        .expect("Could not get viewport image");
    let viewport_tex_id = contexts
        .image_id(&viewport)
        .expect("Could not get viewport texture ID");
    let ctx = contexts.ctx_mut();

    DockArea::new(&mut docktree)
        .style(Style::from_egui(ctx.style().as_ref()))
        .show(
            ctx,
            &mut TabViewer {
                viewport_image,
                viewport_tex_id,
            },
        );
}

fn rotate_cube(time: Res<Time>, mut query: Query<&mut Transform, With<ExampleCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_seconds());
        transform.rotate_z(1.3 * time.delta_seconds());
    }
}
