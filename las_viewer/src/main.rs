use std::f32::consts::PI;

use bevy::{
    color::palettes::basic::SILVER,
    pbr::{wireframe::{WireframeConfig, WireframePlugin}, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef, render_asset::RenderAssetUsages, render_resource::{AsBindGroup, Extent3d, PolygonMode, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, TextureDimension, TextureFormat}
    },
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

const CACHE_PATH: &str = "./cached_points";

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: LinearRgba,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            WireframePlugin,
            PanOrbitCameraPlugin,
            MaterialPlugin::<LineMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        //.add_systems(Update, ())
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    //bevy::pbr::wireframe::WireframeMaterial::default();
    //let debug_material = materials.add(StandardMaterial::from_color(Color::linear_rgba(0., 1., 0., 1.0)));
    let debug_material = line_materials.add(LineMaterial { color: LinearRgba::GREEN });

    let (points, header) = read_las("/home/hey/Downloads/2743_1234.las");
    let bounds = header.bounds();

    let mut modified_points = octotree(&points, &bounds);
    drop(points);

    transform(&mut modified_points, &bounds);

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, modified_points);

    let lines = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        RenderAssetUsages::default(),
    )
    //.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]])
    //.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 1.0], [0.5, 0.0], [1.0, 0.0], [0.5, 1.0]])
    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1]))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 1000.0]]);

    let mesh = meshes.add(mesh);
    let lines = meshes.add(lines);

    commands.spawn((
        PbrBundle {
            mesh: mesh,
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
              .with_rotation(Quat::from_rotation_x(45.0)),
            ..default()
        },
        Shape,
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: lines,
            material: debug_material,
            //material: debug_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
             // .with_rotation(Quat::from_rotation_x(45.0)),
            ..default()
        },
        Shape,
    ));

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

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 7., 14.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    let text_style = TextStyle {
        font_size: 60.0,
        color: Color::WHITE,
        ..Default::default()
    };

    commands.spawn(TextBundle::from_sections([TextSection::new(
        "hello", text_style,
    )]));
}

fn read_las(path: &str) -> (Vec<[f32; 3]>, las::header::Header) {
    
    let mut reader = las::Reader::from_path(path).unwrap();
    let header = las::Read::header(&reader).clone();
    println!("{header:#?}");

    

    let cache_path = std::path::Path::new(CACHE_PATH);
    let points = if cache_path.exists() {
        println!("reading cached points...");
        let points = std::fs::read(CACHE_PATH).expect("faild to read cache file, delete it");
        println!("deserializing cached points...");
        let points = bincode::deserialize::<Vec<[f32; 3]>>(&points).expect("failed to deserialize cache file, delete it");
        println!("reading/deserializing completed");
        points
    } else {
        let mut points = Vec::with_capacity((header.number_of_points() / 3) as usize);

        let mut p_i: usize = 0;

        for wrapped_point in las::Read::points(&mut reader) {
            let point = wrapped_point.unwrap();
    
            points.extend([[point.x as f32, point.y as f32, point.z as f32]]);
    
            if p_i % 100000 == 0 {
                println!("reading points: {}", p_i);
            }
    
            p_i += 1;
        }
        println!("finished reading points: {}", p_i);

        let points_bytes = bincode::serialize(&points).unwrap();
        std::fs::write(CACHE_PATH, points_bytes).unwrap();
        points
    };

   
    let result = (points, header);

    result
}

fn octotree(points: &[[f32; 3]], bounds: &las::Bounds) -> Vec<[f32; 3]> {
    let mut modified_pos = Vec::with_capacity(points.len());
    let mut p_i = 0;

    //let m = bounds.min.z;
    let min_x = bounds.min.x as f32;
    let min_y = bounds.min.y as f32;
    let min_z = bounds.min.z as f32;

    let max_x = bounds.max.x as f32;
    let max_y = bounds.max.y as f32;
    let max_z = bounds.max.z as f32;

    let len_x = max_x - min_x;
    let len_y = max_y - min_y;
    let len_z = max_z - min_z;

    let len_x_middle = len_x / 2.0;
    let len_y_middle = len_y / 2.0;
    let len_z_middle = len_z / 2.0;

    for [x, y, z] in points {
        // if *x > len_x_middle + min_x {
        //     continue;
        // }

        
    //   if
    //   // p_i % 100 == 0 
    //      (*x > len_x_middle + min_x
    //     && *y > len_y_middle + min_y
    //     && *z > len_z_middle + min_z)
    //      ||

    //      (*x < len_x_middle + min_x
    //         && *y > len_y_middle + min_y
    //         && *z > len_z_middle + min_z)
        
    //     {
    //         modified_pos.extend([[*x, *y, *z]]);
    //     }
        modified_pos.extend([[*x, *y, *z]]);

        if p_i % 100000 == 0 {
            println!("octotree points: {}", p_i);
        }
        p_i += 1;
    }

    println!("finished octotree'ing points: {}", p_i);

    modified_pos
}

fn transform(points: &mut [[f32; 3]], bounds: &las::Bounds) {
    let mut p_i = 0;

    for [x, y, z] in points.iter_mut() {
        *x = (*x - bounds.min.x as f32) - ((bounds.max.x as f32 - bounds.min.x as f32) / 2.0);
        *y = (*y - bounds.min.y as f32) - ((bounds.max.y as f32 - bounds.min.y as f32) / 2.0);
        *z -= bounds.min.z as f32;

        if p_i % 100000 == 0 {
            println!("modifying points: {}", p_i);
        }

        p_i += 1;
    }

    println!("finished modifying points: {}", p_i);
}



impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}
