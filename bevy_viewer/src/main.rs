use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::PURPLE,
    dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin},
    image::ImageType,
    input::{
        gestures::{PanGesture, PinchGesture},
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    },
    math::VectorSpace,
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::{
            AsBindGroup, Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use bevy_image_export::ImageExportPlugin;
use binrw::BinRead;
use char::setup_char;
use egui_blocking_plugin::{EguiBlockInputState, EguiBlockingPlugin};
use load::{load_mesh, setup_image, shape_bundle};
use mask::setup_mask;
use std::{f32::consts::*, fs::File, io::BufReader};
use vee::{
    charinfo::nx::NxCharInfo,
    color::cafe::HAIR_COLOR,
    mask::{FacePart, FaceParts},
    shape_load::nx::{ResourceShape, Shape},
    tex_load::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT, TextureElement},
};

mod char;
mod load;
mod mask;

#[derive(Component)]
struct MainPassCamera;

#[derive(Resource, Default)]
struct CharDataRes(Option<ResourceShape>);

#[derive(Resource)]
struct GuiData {
    selected_hair: u32,
    selected_color: u32,
}
impl Default for GuiData {
    fn default() -> Self {
        GuiData {
            selected_hair: 123,
            selected_color: 1,
        }
    }
}

fn main() {
    // let export_plugin = ImageExportPlugin::default();
    // let export_threads = export_plugin.threads.clone();

    App::new()
        .init_resource::<CameraSettings>()
        .init_resource::<CharDataRes>()
        .init_resource::<GuiData>()
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                synchronous_pipeline_compilation: true,
                ..default()
            }),
            EguiBlockingPlugin,
        ))
        .add_systems(Startup, (setup, setup_mask, setup_char).chain())
        .add_systems(Update, (cursor_ungrab, orbit))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(EguiContextPass, ui_example_system)
        .run();

    // export_threads.finish();
}

fn ui_example_system(
    mut contexts: EguiContexts,
    res: Res<CharDataRes>,
    mut gui_data: ResMut<GuiData>,
    char: Single<Entity, With<CharMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) -> Result<()> {
    let res = res.0.unwrap_or(load::get_res()?);

    egui::Window::new("Model Loader").show(contexts.ctx_mut(), |ui| {
        ui.label("Hair Index");
        ui.add(
            egui::DragValue::new(&mut gui_data.selected_hair)
                .speed(0.1)
                .range(0..=res.hair_normal.len() - 1),
        );
        ui.label("Hair Index");
        ui.add(
            egui::DragValue::new(&mut gui_data.selected_color)
                .speed(0.1)
                .range(0..=HAIR_COLOR.len() - 1),
        );

        if ui.button("Load hair model".to_string()).clicked() {
            commands.entity(char.entity()).remove::<(
                Mesh3d,
                MeshMaterial3d<StandardMaterial>,
                Transform,
                CharMesh,
            )>();

            commands.spawn(load::shape_bundle(
                &mut materials,
                &mut meshes,
                &res,
                gui_data.selected_hair as usize,
                gui_data.selected_color as usize,
                Shape::HairNormal,
            ));
        }
    });

    Ok(())
}

#[derive(Component)]
struct CharMesh;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        AmbientLight {
            brightness: 160.0,
            ..default()
        },
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainPassCamera,
    ));
    commands.spawn((
        Name::new("Light"),
        PointLight {
            shadows_enabled: false,
            intensity: 3_000_000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 8.0),
    ));

    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(12.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_translation(vec3(0., -2., 0.)),
    ));

    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "Mouse up or down: pitch\n\
            Mouse left or right: yaw\n\
            Scroll: Zoom in/out\n\
            Escape: use UI\n\
            Click: control camera",
        ),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));
    Ok(())
}

fn cursor_grab(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor_options.visible = false;
}

fn cursor_ungrab(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    egui_block_input_state: Res<EguiBlockInputState>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
) {
    match primary_window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            if egui_block_input_state.wants_pointer_input {
                return;
            }

            if mouse_buttons.pressed(MouseButton::Left) {
                primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor_options.visible = false;
            }
        }
        CursorGrabMode::Confined | CursorGrabMode::Locked => {
            if keyboard_input.pressed(KeyCode::Escape) {
                primary_window.cursor_options.grab_mode = CursorGrabMode::None;
                primary_window.cursor_options.visible = true;
            }
        }
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
    mut camera: Single<&mut Transform, With<MainPassCamera>>,
    mut camera_settings: ResMut<CameraSettings>,
    mut evr_gesture_pinch: EventReader<PinchGesture>,
    mut evr_gesture_pan: EventReader<PanGesture>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    scroll_motion: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    primary_window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if primary_window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }
    let delta = mouse_motion.delta;
    let mut delta_roll = 0.0;

    // if let Some(mut evr_gesture_pinch) = evr_gesture_pinch {
    for ev_pinch in evr_gesture_pinch.read() {
        camera_settings.orbit_distance -= ev_pinch.0 * 10.0;
    }
    // }
    camera_settings.orbit_distance += scroll_motion.delta.y;
    camera_settings.orbit_distance = camera_settings.orbit_distance.clamp(1.0, 1000.0);

    let mut pan = vec2(0.0, 0.0);
    // if let Some(mut evr_gesture_pan) = evr_gesture_pan {
    for ev_pan in evr_gesture_pan.read() {
        pan.x += ev_pan.0.x;
        pan.y += ev_pan.0.y;
    }
    // }
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
    let target = Vec3::ZERO.with_y(1.0);
    camera.translation =
        target - camera.forward() * camera_settings.orbit_distance + pan.extend(0.0);
}

mod egui_blocking_plugin {
    use bevy::prelude::*;
    use bevy_egui::EguiContexts;

    pub struct EguiBlockingPlugin;

    #[derive(Default, Resource)]
    pub struct EguiBlockInputState {
        pub wants_keyboard_input: bool,
        pub wants_pointer_input: bool,
    }

    impl Plugin for EguiBlockingPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<EguiBlockInputState>()
                .add_systems(PostUpdate, egui_wants_input);
        }
    }

    fn egui_wants_input(mut state: ResMut<EguiBlockInputState>, mut contexts: EguiContexts) {
        let ctx = contexts.ctx_mut();
        state.wants_keyboard_input = ctx.wants_keyboard_input();
        state.wants_pointer_input = ctx.wants_pointer_input();
    }
}
