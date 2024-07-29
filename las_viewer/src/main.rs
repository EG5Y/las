use std::{collections::VecDeque, f32::consts::PI};

use bevy::{
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        MaterialPipeline, MaterialPipelineKey,
    },
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, Extent3d, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat,
        },
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
        .run();
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    let debug_material = line_materials.add(LineMaterial {
        color: LinearRgba::GREEN,
    });

    let (mut points, header) = read_las("2743_1234.las");
    let bounds = header.bounds();

    let size = [
        bounds.max.x - bounds.min.x,
        bounds.max.y - bounds.min.y,
        bounds.max.z - bounds.min.z,
    ]
    .into_iter()
    .max_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap() as f32;
    println!("octree size: {}", size);

    let mut tree = octree::Octree::new(size);
    println!("TREE {:#?}", tree);
    let lines = gen_debug_lines(&tree);

    
    transform_move(&mut points, &bounds);
    println!("importing data to octree...");
    tree.import(&points);

    println!("exporting data from octree...");
    let modified_points = tree.export();

    drop(points);

    let mesh: Mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, modified_points);

    

    let mesh = meshes.add(mesh);
    let lines = meshes.add(lines);
    //let sphere = meshes.add(sphere);

    commands.spawn((
        PbrBundle {
            mesh: mesh,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Shape,
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: lines,
            material: debug_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Shape,
    ));

   
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-500.0, 500., -500.0)
                .looking_at(Vec3::new(500., 500., 0.), Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));

}

fn read_las(path: &str) -> (Vec<[f32; 3]>, las::header::Header) {
    let mut reader = las::Reader::from_path(path).expect("las file not found, provide 2743_1234.las in project directory");
    let header = las::Read::header(&reader).clone();
    println!("{header:#?}");

    let cache_path = std::path::Path::new(CACHE_PATH);
    let points = if cache_path.exists() {
        println!("reading cached points...");
        let points = std::fs::read(CACHE_PATH).expect("faild to read cache file, delete it");
        println!("deserializing cached points...");
        let points = bincode::deserialize::<Vec<[f32; 3]>>(&points)
            .expect("failed to deserialize cache file, delete it");
        println!("reading/deserializing completed");
        points
    } else {
        let mut points = Vec::with_capacity((header.number_of_points() / 3) as usize);

        let mut p_i: usize = 0;

        for wrapped_point in las::Read::points(&mut reader) {
            let point = wrapped_point.unwrap();

            points.extend([[point.x as f32, point.z as f32, point.y as f32]]);

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


fn transform_center(points: &mut [[f32; 3]], bounds: &las::Bounds) {
    println!("transforming points...");
    for [x, y, z] in points.iter_mut() {
        *x = (*x - bounds.min.x as f32) - ((bounds.max.x as f32 - bounds.min.x as f32) / 2.0);
        *y = (*y - bounds.min.y as f32) - ((bounds.max.y as f32 - bounds.min.y as f32) / 2.0);
        *z -= bounds.min.z as f32;
    }
    println!("finished transforming points.");
}

fn transform_move(points: &mut [[f32; 3]], bounds: &las::Bounds) {
    println!("transforming points...");

    for [x, y, z] in points.iter_mut() {
        *x -= bounds.min.x as f32;
        *y -= bounds.min.z as f32;
        *z -= bounds.min.y as f32;
    }

    println!("finished transforming points.");
}

fn gen_debug_lines(tree: &octree::Octree) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut queue = VecDeque::<&octree::OctreeNode>::new();

    queue.push_back(&tree.root);

    let mut i: u32 = 0;
    loop {
        let Some(node) = queue.pop_front() else {
            break;
        };

        match node {
            octree::OctreeNode::Cube(cube) => {
                #[rustfmt::skip]
                let new_indices: [u32; 24]  = [
                    i, i + 1,
                    i + 1, i + 2,
                    i + 2, i + 3,
                    i, i + 3,
            
                    i, i + 4,
                    i + 1, i + 5,
                    i + 2, i + 6,
                    i + 3, i + 7,
            
                    i + 4, i + 5,
                    i + 5, i + 6,
                    i + 6, i + 7,
                    i + 4, i + 7,
                ];
        
                pos.extend_from_slice(&cube.cube_points);
                indices.extend(new_indices);

                i += 8;
            }
            octree::OctreeNode::Sphere(sphere) => {

                #[rustfmt::skip]
                let new_indices: [u32; 24]  = [
                    i, i + 1,
                    i + 1, i + 2,
                    i + 2, i + 3,
                    i, i + 3,
            
                    i, i + 4,
                    i + 1, i + 5,
                    i + 2, i + 6,
                    i + 3, i + 7,
            
                    i + 4, i + 5,
                    i + 5, i + 6,
                    i + 6, i + 7,
                    i + 4, i + 7,
                ];

                pos.extend_from_slice(&sphere.cube_points);
                indices.extend(new_indices);
                i += 8;

                #[rustfmt::skip]
                let new_indices = vec![
                    i, i + 1,
                    i + 1, i + 2,
                    i + 2, i + 3,
                    i + 3, i + 4,
                    i, i + 4,
            
                    i, i + 5,
                    i + 1, i + 5,
                    i + 2, i + 5,
                    i + 3, i + 5,
                    i + 4, i + 5,
            
                    i + 6, i + 7,
                    i + 7, i + 8,
                    i + 8, i + 9,
                    i + 9, i + 10,
                    i + 6, i + 10,
            
                    i + 6, i + 11,
                    i + 7, i + 11,
                    i + 8, i + 11,
                    i + 9, i + 11,
                    i + 10, i + 11,
            
                    i, i + 6,
                    i, i + 10,
            
                    i + 1, i + 6,
                    i + 1, i + 7,
            
                    i + 2, i + 7,
                    i + 2, i + 8,
            
                    i + 3, i + 8,
                    i + 3, i + 9,
            
                    i + 4, i + 9,
                    i + 4, i + 10,
                ];

                pos.extend_from_slice(&sphere.sphere_points);
                indices.extend(new_indices);

                i += 12;
            }
        }

       

        if let Some(nodes) = node.nodes_ref() {
            for node in nodes.iter() {
                queue.push_back(node);
            }
        }

        
    }


    let lines = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos);

    lines
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
