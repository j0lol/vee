use bevy::{
    asset::RenderAssetUsages,
    input::mouse::AccumulatedMouseMotion,
    pbr::PbrPlugin,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use binrw::{BinRead, io::BufReader};
use std::fs::File;
use std::{default, f32::consts::*};
use vee::{ResourceShape, ShapeData};

#[derive(Resource, Default)]
struct MiiDataRes(Option<ResourceShape>);

#[derive(Resource, Default)]
struct GuiData {
    selected_hair: u32,
}

fn main() {
    App::new()
        .init_resource::<CameraSettings>()
        .init_resource::<MiiDataRes>()
        .init_resource::<GuiData>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, cursor_grab).chain())
        .add_systems(Update, (orbit, cursor_ungrab))
        // .add_plugins(EguiPlugin {
        //     enable_multipass_for_primary_context: true,
        // })
        // .add_systems(EguiContextPass, ui_example_system)
        .run();
}

fn shape_data_to_mesh(data: ShapeData) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
    .with_inserted_indices(Indices::U16(data.indices))
}
fn load_mesh(res: ResourceShape, hair_num: usize) -> Result<Mesh> {
    let mut shape = res.hair_normal[hair_num];
    let mut file = File::open("ShapeMid.dat")?;
    let mesh = shape.shape_data(&mut file).unwrap();

    // let gltf = shape.gltf(&mut file).unwrap();
    // gltf.export_glb("assets/model.glb")?;

    Ok(shape_data_to_mesh(mesh))
}

fn get_res() -> Result<ResourceShape> {
    let mut bin = BufReader::new(File::open("ShapeMid.dat")?);
    Ok(ResourceShape::read(&mut bin)?)
}
fn ui_example_system(
    mut contexts: EguiContexts,
    res: Res<MiiDataRes>,
    mut gui_data: ResMut<GuiData>,
    mut mii: Single<(&Mesh3d, &MeshMaterial3d<StandardMaterial>, Entity), With<MiiMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) -> Result<()> {
    let res = res.0.unwrap_or(get_res()?);

    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.add(
            egui::DragValue::new(&mut gui_data.selected_hair)
                .speed(0.1)
                .range(0..=res.hair_normal.len() - 1),
        );

        if ui.button("Load hair model".to_string()).clicked() {
            // mesh.0. ;
            meshes
                .get_mut(mii.0.id())
                .replace(&mut load_mesh(res, gui_data.selected_hair as usize).unwrap());

            commands.spawn((
                Mesh3d(mii.0.0.clone()),
                MeshMaterial3d(mii.1.0.clone()),
                Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.1)),
                MiiMesh,
            ));
            commands
                .entity(mii.2)
                .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform, MiiMesh)>();
        }
        //
    });

    Ok(())
}

#[derive(Component)]
struct MiiMesh;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut res: ResMut<MiiDataRes>,
) -> Result<()> {
    res.0 = Some(get_res()?);

    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(load_mesh(res.0.unwrap(), 123)?);

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        Mesh3d(cube_mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.118, 0.102, 0.094),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.1)),
        MiiMesh,
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        // AmbientLight::default(),
    ));
    commands.spawn((
        Name::new("Light"),
        PointLight {
            intensity: 1000.0,
            color: Color::srgb(1.0, 1.0, 0.7),
            ..default()
        },
        Transform::from_xyz(3.0, 80.0, 5.0),
    ));

    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "Mouse up or down: pitch\n\
            Mouse left or right: yaw\n\
            Mouse buttons: roll",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));
    Ok(())
}

fn cursor_grab(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    // if you want to use the cursor, but not let it leave the window,
    // use `Confined` mode:

    primary_window.cursor_options.grab_mode = CursorGrabMode::Confined;

    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor_options.visible = false;
}

fn cursor_ungrab(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        primary_window.cursor_options.grab_mode = CursorGrabMode::None;
        primary_window.cursor_options.visible = true;
    }
}
#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    // Clamp pitch to this range
    pub pitch_range: std::ops::Range<f32>,
    pub roll_speed: f32,
    pub yaw_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            // These values are completely arbitrary, chosen because they seem to produce
            // "sensible" results for this example. Adjust as required.
            orbit_distance: 20.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            roll_speed: 1.0,
            yaw_speed: 0.004,
        }
    }
}
fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    let delta = mouse_motion.delta;
    let mut delta_roll = 0.0;

    if mouse_buttons.pressed(MouseButton::Left) {
        delta_roll -= 1.0;
    }
    if mouse_buttons.pressed(MouseButton::Right) {
        delta_roll += 1.0;
    }

    // Mouse motion is one of the few inputs that should not be multiplied by delta time,
    // as we are already receiving the full movement since the last frame was rendered. Multiplying
    // by delta time here would make the movement slower that it should be.
    let delta_pitch = delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;

    // Conversely, we DO need to factor in delta time for mouse button inputs.
    delta_roll *= camera_settings.roll_speed * time.delta_secs();

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );
    let roll = roll + delta_roll;
    let yaw = yaw + delta_yaw;
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    // In our example it's a static target, but this could easily be customized.
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
}
