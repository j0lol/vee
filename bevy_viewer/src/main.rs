use bevy::{
    asset::RenderAssetUsages,
    dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin},
    image::ImageType,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::{
        render_resource::{Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_egui::{EguiContextPass, EguiContexts, EguiPlugin, egui};
use binrw::BinRead;
use egui_blocking_plugin::{EguiBlockInputState, EguiBlockingPlugin};
use load::{load_mesh, setup_image, shape_bundle};
use std::{f32::consts::*, fs::File, io::BufReader};
use vee::{
    charinfo::nx::NxCharInfo,
    color::cafe::HAIR_COLOR,
    mask::FaceParts,
    shape_load::nx::{ResourceShape, Shape},
    tex_load::nx::{ResourceTexture, TEXTURE_MID_SRGB_DAT},
};

mod load;

#[derive(Component)]
struct MainPassCamera;

#[derive(Resource, Default)]
struct MiiDataRes(Option<ResourceShape>);

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
    App::new()
        .init_resource::<CameraSettings>()
        .init_resource::<MiiDataRes>()
        .init_resource::<GuiData>()
        .add_plugins((MeshPickingPlugin, DebugPickingPlugin))
        .insert_resource(DebugPickingMode::Normal)
        .add_plugins((DefaultPlugins, EguiBlockingPlugin))
        .add_systems(Startup, (setup, setup_rendtotexexample).chain())
        .add_systems(Update, (cube_rotator_system, rotator_system))
        .add_systems(Update, (cursor_ungrab, orbit))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(EguiContextPass, ui_example_system)
        .run();
}

fn ui_example_system(
    mut contexts: EguiContexts,
    res: Res<MiiDataRes>,
    mut gui_data: ResMut<GuiData>,
    mii: Single<Entity, With<MiiMesh>>,
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
            commands.entity(mii.entity()).remove::<(
                Mesh3d,
                MeshMaterial3d<StandardMaterial>,
                Transform,
                MiiMesh,
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
struct MiiMesh;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut res: ResMut<MiiDataRes>,
) -> Result<()> {
    res.0 = Some(load::get_res()?);

    let mut mii = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../j0.charinfo")).unwrap();

    let mii = NxCharInfo::read(&mut mii).unwrap();

    // let image = {
    //     //
    //     let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?);

    //     let res = ResourceTexture::read(&mut bin)?;

    //     let tex = res.eye[0]
    //         .get_texture(&mut BufReader::new(File::open(TEXTURE_MID_SRGB_DAT)?))
    //         .unwrap();

    //     Image::new(
    //         Extent3d {
    //             width: res.eye[0].texture.width.into(),
    //             height: res.eye[0].texture.height.into(),
    //             depth_or_array_layers: 1,
    //         },
    //         bevy::render::render_resource::TextureDimension::D2,
    //         tex,
    //         TextureFormat::Bc7RgbaUnormSrgb,
    //         RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    //     )
    // };

    // commands.spawn(shape_bundle(
    //     &mut materials,
    //     &mut meshes,
    //     &res.0.unwrap(),
    //     1,
    //     4,
    //     Shape::Mask,
    // ));
    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        1,
        4,
        Shape::FaceLine,
    ));

    // Create and save a handle to the mesh.
    // Render the mesh with the custom texture, and add the marker.
    commands.spawn(shape_bundle(
        &mut materials,
        &mut meshes,
        &res.0.unwrap(),
        mii.hair_type as usize,
        mii.hair_color as usize,
        Shape::HairNormal,
    ));

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

    camera_settings.orbit_distance += scroll_motion.delta.y;
    camera_settings.orbit_distance = camera_settings.orbit_distance.clamp(1.0, 1000.0);

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

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn draw_mii_mask(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands,
    render_layer: RenderLayers,
) {
    let mut mii = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../j0.charinfo")).unwrap();

    let mii = NxCharInfo::read(&mut mii).unwrap();

    let mask_info = FaceParts::init(&mii, 256.0);

    let eye_pos = mask_info.eye[0];

    let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());

    let vee_textures = ResourceTexture::read(&mut bin).unwrap();

    let tex = vee_textures.eye[mii.eye_type as usize];

    let quad_handle = meshes.add(Rectangle::new(
        f32::from(tex.texture.width) / 256.0 * 2.,
        f32::from(tex.texture.height) / 256.0 * 2.,
    ));

    let tex = tex
        .get_image(&mut BufReader::new(
            File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
        ))
        .unwrap()
        .unwrap();

    let img = setup_image(images, tex);

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(img.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // textured quad - normal
    commands.spawn((
        Mesh3d(quad_handle.clone()),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(eye_pos.x / 256.0, eye_pos.y / 256.0, 1.0),
        render_layer.clone(),
    ));

    // load mouth

    let mouth_pos = mask_info.mouth;

    let mut bin = BufReader::new(File::open(TEXTURE_MID_SRGB_DAT).unwrap());

    let vee_textures = ResourceTexture::read(&mut bin).unwrap();

    let tex = vee_textures.mouth[mii.mouth_type as usize];

    let quad_handle = meshes.add(Rectangle::new(
        f32::from(tex.texture.width) / 256.0 * 2.,
        f32::from(tex.texture.height) / 256.0 * 2.,
    ));

    let tex = tex
        .get_image(&mut BufReader::new(
            File::open(TEXTURE_MID_SRGB_DAT).unwrap(),
        ))
        .unwrap()
        .unwrap();

    let img = setup_image(images, tex);

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(img.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // textured quad - normal
    commands.spawn((
        Mesh3d(quad_handle.clone()),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(mouth_pos.x / 256.0, mouth_pos.y / 256.0, 1.0),
        render_layer,
    ));
}

fn setup_rendtotexexample(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut res: ResMut<MiiDataRes>,
) {
    res.0 = Some(load::get_res().unwrap());

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    // You need to set these texture usage flags in order to use the image as a render target
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);

    let cube_handle = meshes.add(Cuboid::new(4.0, 4.0, 4.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = RenderLayers::layer(1);

    // The cube that will be rendered to the texture.
    // commands.spawn((
    //     Mesh3d(cube_handle),
    //     MeshMaterial3d(cube_material_handle),
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    //     FirstPassCube,
    //     first_pass_layer.clone(),
    // ));

    draw_mii_mask(
        &mut materials,
        &mut meshes,
        &mut images,
        &mut commands,
        first_pass_layer.clone(),
    );

    // Light
    // NOTE: we add the light to both layers so it affects both the rendered-to-texture cube, and the cube on which we display the texture
    // Setting the layer to RenderLayers::layer(0) would cause the main view to be lit, but the rendered-to-texture cube to be unlit.
    // Setting the layer to RenderLayers::layer(1) would cause the rendered-to-texture cube to be lit, but the main view to be unlit.
    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        RenderLayers::layer(1),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection::default()),
        Camera {
            target: image_handle.clone().into(),
            clear_color: Color::NONE.into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)).looking_at(Vec3::ZERO, Vec3::Y),
        first_pass_layer,
    ));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    commands.spawn({
        let meshes: &mut ResMut<Assets<Mesh>> = &mut meshes;
        let res: &ResourceShape = &res.0.unwrap();
        let shape = Shape::Mask;

        (
            Mesh3d(meshes.add(load_mesh(*res, shape, 1).unwrap())),
            MeshMaterial3d(material_handle),
            Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(0.05)),
        )
    });

    // Main pass cube, with material containing the rendered first pass texture.
    // commands.spawn((
    //     Mesh3d(cube_handle),
    //     MeshMaterial3d(material_handle),
    //     Transform::from_xyz(0.0, 0.0, 1.5).with_rotation(Quat::from_rotation_x(-PI / 5.0)),
    //     MainPassCube,
    // ));

    // The main pass camera.
    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}

/// Rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<FirstPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.5 * time.delta_secs());
        transform.rotate_z(1.3 * time.delta_secs());
    }
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.0 * time.delta_secs());
        transform.rotate_y(0.7 * time.delta_secs());
    }
}
