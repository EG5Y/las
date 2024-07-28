use std::{collections::VecDeque, f32::consts::PI};

use bevy::{
    color::palettes::basic::SILVER,
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

#[derive(Debug, Clone)]
struct Octree {
    root: OctreeCube, // pub points: [[f32; 3]; 8],
                      // nodes: [OctreeCube; 8]
}

#[derive(Debug, Clone)]
struct OctreeCube {
    points: [[f32; 3]; 8],
    nodes: Option<Box<[OctreeCube; 8]>>,
}

impl Octree {
    pub fn new(size: f32) -> Self {
        let root = octree_gen_points(0.0, 0.0, 0.0, size);

        
        let mut nodes = OctreeCube::into_cluster(root);
        nodes[0].divide();
        nodes[0].nodes.as_mut().unwrap()[0].divide();


        Self {
            root: OctreeCube {
                points: root,
                nodes: Some(Box::new(nodes)),
            },
        }
    }
}

impl OctreeCube {
    pub fn divide(&mut self) {
        self.nodes = Some(Box::new(OctreeCube::into_cluster(self.points)));
    }

    pub fn into_cluster(cube: [[f32; 3]; 8]) -> [OctreeCube; 8] {
        let cluster = octree_divide(&cube);
        OctreeCube::from_cluster(cluster)
    }

    pub fn from_cluster(points: [[[f32; 3]; 8]; 8]) -> [OctreeCube; 8] {
        [
            OctreeCube { points: points[0], nodes: None },
            OctreeCube { points: points[1], nodes: None },
            OctreeCube { points: points[2], nodes: None },
            OctreeCube { points: points[3], nodes: None },

            OctreeCube { points: points[4], nodes: None },
            OctreeCube { points: points[5], nodes: None },
            OctreeCube { points: points[6], nodes: None },
            OctreeCube { points: points[7], nodes: None },
        ]
    }
}

fn octree_gen_points(x: f32, y: f32, z: f32, size: f32) -> [[f32; 3]; 8] {
    [
        [x, y, z],
        [x, y+size, z],
        [x+size, y+size, z],
        [x+size, y, z],
        [x, y, z+size],
        [x, y+size, z+size],
        [x+size, y+size, z+size],
        [x+size, y, z+size],
    ]
}

fn octree_divide(octree_cube: &[[f32; 3]; 8]) -> [[[f32; 3]; 8]; 8] {
    let [start_x, start_y, start_z] = octree_cube[0];
    let [width_x, width_y, width_z] = octree_cube[3];
    let [height_x, height_y, height_z] = octree_cube[1];
    let [len_x, len_y, len_z] = octree_cube[4];

    let size = (width_x - start_x) / 2.0;
    let middle_x = start_x + size;
    let middle_y = start_y + size;
    let middle_z = start_z + size;

    // let middle_x = start_x + (width_x - start_x) / 2.0;
    // let middle_y = start_y + (height_y - start_y) / 2.0;
    // let middle_z = start_z + (len_z - start_z) / 2.0;

    // let [width_half_x, width_half_y, width_half_z] = [
    //     start_x + (width_x - start_x) / 2.0,
    //     start_y + (width_y),
    //     width_z / 2.0,
    // ];
    // let [height_half_x, height_half_y, height_half_z] =
    //     [height_x / 2.0, height_y / 2.0, height_z / 2.0];
    // let [len_half_x, len_half_y, len_half_z] = [len_x / 2.0, len_y / 2.0, len_z / 2.0];

    [
        octree_gen_points(start_x, start_y, start_z, size),
        octree_gen_points(start_x, middle_y, start_z, size),
        octree_gen_points(middle_x, middle_y, start_z, size),
        octree_gen_points(middle_x, start_y, start_z, size),

        octree_gen_points(start_x, start_y, middle_z, size),
        octree_gen_points(start_x, middle_y, middle_z, size),
        octree_gen_points(middle_x, middle_y, middle_z, size),
        octree_gen_points(middle_x, start_y, middle_z, size),

    ]
}

// impl Octree {
//     pub fn new(height: f32, width: f32, length: f32) -> Self {
//         Self {
//             points: [
//                 [0.0, 0.0, 0.0],
//                 [0.0, height, 0.0],
//                 [width, height, 0.0],
//                 [width, 0.0, 0.0],
//                 [0.0, 0.0, length],
//                 [0.0, height, length],
//                 [width, height, length],
//                 [width, 0.0, length],
//             ],
//         }
//     }
// }

enum OctreeNode {
    Sphere,
    Cube,
}

struct OctreeSphere {}

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
    let debug_material = line_materials.add(LineMaterial {
        color: LinearRgba::GREEN,
    });

    let (points, header) = read_las("/home/hey/Downloads/2743_1234.las");
    let bounds = header.bounds();

    let mut modified_points = octotree(&points, &bounds);
    drop(points);

    transform_move(&mut modified_points, &bounds);

    let mesh: Mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, modified_points);

    let size = [
        bounds.max.x - bounds.min.x,
        bounds.max.y - bounds.min.y,
        bounds.max.z - bounds.min.z,
    ]
    .into_iter()
    .max_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap() as f32;
    println!("octree size: {}", size);
    //let size = [bounds.max.x, bounds.max.y , bounds.max.z].into_iter().max().unwrap() as f32;

    //max!

    let tree = Octree::new(size);
    println!("{:#?}", tree);
    let lines = gen_debug_lines(&tree);

    let mesh = meshes.add(mesh);
    let lines = meshes.add(lines);

    commands.spawn((
        PbrBundle {
            mesh: mesh,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            //  .with_rotation(Quat::from_rotation_x(45.0)),
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

fn octotree(points: &[[f32; 3]], bounds: &las::Bounds) -> Vec<[f32; 3]> {
    let mut modified_pos = Vec::with_capacity(points.len());

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
    println!("running octree algo...");
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
    }

    println!("finished octree algo");

    modified_pos
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

fn gen_debug_lines(tree: &Octree) -> Mesh {
    //let pos = tree.root.points.to_vec();
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut queue = VecDeque::<&OctreeCube>::new();

    queue.push_back(&tree.root);
    // if let Some(nodes) = &tree.root.nodes {
    //     //queue.push_back(&nodes[0]) ;
    //     for node in nodes.iter().skip(1) {
    //         queue.push_back(node);
    //     }
    // }
    

    let mut i: u32 = 0;
    loop {
        let Some(cube) = queue.pop_front() else {
            break;
        };

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

        pos.extend_from_slice(&cube.points);
        indices.extend(new_indices);

        if let Some(nodes) = &cube.nodes {
            for node in nodes.iter() {
                queue.push_back(node);
            }
        }

        i += 8;
    }

    
   // let indices = vec!;

    let lines = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        RenderAssetUsages::default(),
    )
    //.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]])
    //.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 1.0], [0.5, 0.0], [1.0, 0.0], [0.5, 1.0]])
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
