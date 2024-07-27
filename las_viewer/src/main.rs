//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

use std::f32::consts::PI;

use bevy::{
    color::palettes::basic::SILVER,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WireframePlugin,
            bevy_flycam::NoCameraPlayerPlugin,
            bevy_fsc_point_cloud::PointCloudPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_wireframe))
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // let shapes = [
    //     meshes.add(Cuboid::default()),
    //     meshes.add(Tetrahedron::default()),
    //     meshes.add(Capsule3d::default()),
    //     meshes.add(Torus::default()),
    //     meshes.add(Cylinder::default()),
    //     meshes.add(Cone::default()),
    //     meshes.add(ConicalFrustum::default()),
    //     meshes.add(Sphere::default().mesh().ico(5).unwrap()),
    //     meshes.add(Sphere::default().mesh().uv(32, 18)),
    // ];

    // let extrusions = [
    //     meshes.add(Extrusion::new(Rectangle::default(), 1.)),
    //     meshes.add(Extrusion::new(Capsule2d::default(), 1.)),
    //     meshes.add(Extrusion::new(Annulus::default(), 1.)),
    //     meshes.add(Extrusion::new(Circle::default(), 1.)),
    //     meshes.add(Extrusion::new(Ellipse::default(), 1.)),
    //     meshes.add(Extrusion::new(RegularPolygon::default(), 1.)),
    //     meshes.add(Extrusion::new(Triangle2d::default(), 1.)),
    // ];

    // let num_shapes = shapes.len();

    // for (i, shape) in shapes.into_iter().enumerate() {
    //     commands.spawn((
    //         PbrBundle {
    //             mesh: shape,
    //             material: debug_material.clone(),
    //             transform: Transform::from_xyz(
    //                 -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
    //                 2.0,
    //                 Z_EXTENT / 2.,
    //             ),
    //             //.with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //             ..default()
    //         },
    //         Shape,
    //     ));
    // }

    //let num_extrusions = extrusions.len();

    // let cube = Extrusion::new(Rectangle::default(), 1.);
    // let cube1 = meshes.add(Cuboid::default());

    // commands.spawn((
    //     PbrBundle {
    //         mesh: cube1.clone(),
    //         material: debug_material.clone(),
    //         transform: Transform::from_xyz(
    //             0.0,
    //             0.0,
    //             0.0,
    //         ),
            
    //        // .with_rotation(Quat::ro),
    //        // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //         ..default()
    //     },
    //     Shape,
    // ));

    // commands.spawn((
    //     PbrBundle {
    //         mesh: cube1.clone(),
    //         material: debug_material.clone(),
    //         transform: Transform::from_xyz(
    //             0.0,
    //             1.0,
    //             0.0,
    //         ),
            
    //        // .with_rotation(Quat::ro),
    //        // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //         ..default()
    //     },
    //     Shape,
    // ));

    // let mut pos = vec![[-1.0, 0.0, 0.0], [-1.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, 0.0]];
    // let mut norm = vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]];
    // let mut index = vec![
    //     // First triangle
    //     0, 3, 1,
    //     // Second triangle
    //     1, 3, 2
    // ];

    let mut i: u32 = 0;
    let mut pos = Vec::new();
    let mut uv = Vec::new();
    let mut norm = Vec::new();
    let mut index: Vec<u32> = Vec::new();

    let mut add_quad = |x: f32, y: f32, z: f32| {
        pos.extend([[x-1.0, y, z], [x-1.0, y, z-1.0], [x, y, z-1.0], [x, y, z]]);
        uv.extend([[0.0, 1.0], [0.5, 0.0], [1.0, 0.0], [0.5, 1.0]]);
        norm.extend([[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]);
        index.extend([
            i, 3 + i, 
            1 + i, 
            1 + i, 
            3 + i, 
            2 + i,
        ]);
        i += 4;
    };


    // let mut reader = las::Reader::from_path("/home/hey/Downloads/2743_1234.las").unwrap();
    // //let point = las::Read::read(&mut reader).unwrap().unwrap();
    // for wrapped_point in las::Read::points(&mut reader) {
    //     let point = wrapped_point.unwrap();

    //     add_quad(point.x as f32, point.y as f32, point.z as f32);

    //     // commands.spawn((
    //     //     PbrBundle {
    //     //         mesh: cube1.clone(),
    //     //         material: debug_material.clone(),
    //     //         transform: Transform::from_xyz(
    //     //             point.x as f32,
    //     //             point.y as f32,
    //     //             point.z as f32,
    //     //         ),
                
    //     //        // .with_rotation(Quat::ro),
    //     //        // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //     //         ..default()
    //     //     },
    //     //     Shape,
    //     // ));

    //     //println!("Point coordinates: ({}, {}, {})", point.x, point.y, point.z);
    //     // if let Some(color) = point.color {
    //     //     println!("Point color: red={}, green={}, blue={}",
    //     //         color.red,
    //     //         color.green,
    //     //         color.blue,
    //     //     );
    //     // }
    // }


    
    add_quad(1.0, 2.0, 0.0);


    let mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
    // Add 4 vertices, each with its own position attribute (coordinate in
    // 3D space), for each of the corners of the parallelogram.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        pos
    )
    // Assign a UV coordinate to each vertex.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        uv
    )
    // Assign normals (everything points outwards)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        norm,
    )
    // After defining all the vertices and their attributes, build each triangle using the
    // indices of the vertices that make it up in a counter-clockwise order.
    .with_inserted_indices(bevy::render::mesh::Indices::U32(index));

    let mesh = meshes.add(mesh);

    commands.spawn((
        PbrBundle {
            mesh: mesh,
            material: debug_material.clone(),
            transform: Transform::from_xyz(
                0.0,
                1.0,
                0.0,
            ),
            
           // .with_rotation(Quat::ro),
           // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            ..default()
        },
        Shape,
    ));


    let mesh: Handle<PointCloudAsset> = asset_server.load("/home/hey/Downloads/2743_1234.las");

    commands
        .spawn(PotreePointCloud {
            mesh,
            point_size: 0.007,
        })
        .insert(SpatialBundle::default());


    // let mut reader = las::Reader::from_path("/home/hey/Downloads/2743_1234.las").unwrap();
    // let point = las::Read::read(&mut reader).unwrap().unwrap();
    // for wrapped_point in las::Read::points(&mut reader) {
    //     let point = wrapped_point.unwrap();

    //     commands.spawn((
    //         PbrBundle {
    //             mesh: cube1.clone(),
    //             material: debug_material.clone(),
    //             transform: Transform::from_xyz(
    //                 point.x as f32,
    //                 point.y as f32,
    //                 point.z as f32,
    //             ),
                
    //            // .with_rotation(Quat::ro),
    //            // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //             ..default()
    //         },
    //         Shape,
    //     ));

    //     //println!("Point coordinates: ({}, {}, {})", point.x, point.y, point.z);
    //     // if let Some(color) = point.color {
    //     //     println!("Point color: red={}, green={}, blue={}",
    //     //         color.red,
    //     //         color.green,
    //     //         color.blue,
    //     //     );
    //     // }
    // }

    // for (i, shape) in extrusions.into_iter().enumerate() {
    //     commands.spawn((
    //         PbrBundle {
    //             mesh: shape,
    //             material: debug_material.clone(),
    //             transform: Transform::from_xyz(
    //                 -EXTRUSION_X_EXTENT / 2.
    //                     + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
    //                 2.0,
    //                 -Z_EXTENT / 2.,
    //             ),
                
    //            // .with_rotation(Quat::ro),
    //            // .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    //             ..default()
    //         },
    //         Shape,
    //     ));
    // }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10)),
    //     material: materials.add(Color::from(SILVER)),
    //     ..default()
    // });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        bevy_flycam::FlyCam,
    ));

    // commands.spawn(
    //     TextBundle::from_section("Press space to toggle wireframes", TextStyle::default())
    //         .with_style(Style {
    //             position_type: PositionType::Absolute,
    //             top: Val::Px(12.0),
    //             left: Val::Px(12.0),
    //             ..default()
    //         }),
    // );
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}