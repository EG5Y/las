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
    root: OctreeNode, // pub points: [[f32; 3]; 8],
                      // nodes: [OctreeCube; 8]
}

#[derive(Debug, Clone)]
struct OctreeCube {
    pub cube_points: [[f32; 3]; 8],
    pub nodes: Option<Box<[OctreeNode; 8]>>,
}

#[derive(Debug, Clone)]
struct OctreeSphere {
    pub middle: [f32; 3],
    pub radius: f32,
    pub data_points: Vec<[f32; 3]>,
    pub cube_points: [[f32; 3]; 8],
    pub sphere_points: [[f32; 3]; 12],
    pub nodes: Option<Box<[OctreeNode; 8]>>,
}

#[derive(Debug, Clone)]
enum OctreeNode {
    Cube(OctreeCube),
    Sphere(OctreeSphere),
}

impl Octree {
    pub fn new(size: f32) -> Self {
        let root = gen_cube(0.0, 0.0, 0.0, size);

        
        let mut nodes = OctreeCube::into_cluster(root);
        nodes[0].divide();
        
        nodes[0].node(0).divide_into_spheres();
        nodes[0].node(0).node(0).divide_into_cubes();
        nodes[0].node(1).divide_into_cubes();
        nodes[0].node(1).node(0).divide_into_spheres();

        Self {
            root: OctreeNode::Cube(
                OctreeCube {
                    cube_points: root,
                    nodes: Some(Box::new(nodes)),
                }
            ),
        }
    }

    pub fn import(&mut self, data: &[[f32; 3]]) {
        self.root.import(data);
        // let mut queue = VecDeque::<&mut OctreeNode>::new();

        // queue.push_back(&mut self.root);

        // loop {
        //     let Some(node) = queue.pop_front() else {
        //         break;
        //     };

        //     let last_node = false;
            
        //     {
        //         if let Some(nodes) = node.nodes_mut() {
        //             for node in nodes.iter_mut() {
        //                 queue.push_back(node);
        //             }
        //         } else {
        //         }
        //     }

        //     node.divide();

        //     // {
        //     //     if last_node {
        //     //         match node {
        //     //             OctreeNode::Cube(cube) => {
    
        //     //             }
        //     //             OctreeNode::Sphere(sphere) => {
        //     //                 for data_point in data {
        //     //                     let is_inside = sphere.is_inside(data_point);
        //     //                     if is_inside {
        //     //                         sphere.data_points.push(*data_point);
        //     //                     }
        //     //                 }
        //     //             }
        //     //         }
        //     //     }
        //     // }

           
        // }

        
    }

    pub fn export(&self) -> Vec<[f32; 3]> {
        let mut output = Vec::new();
        self.root.export(&mut output);
        output
    }
}

impl OctreeNode {
    pub fn divide(&mut self) {
        match self {
            OctreeNode::Cube(cube) => {
                cube.divide();
            }
            OctreeNode::Sphere(sphere) => {
                sphere.divide();
            }
        }
    }

    pub fn divide_into_cubes(&mut self) {
        let cube_points = self.cube_points();
        let cluster = OctreeCube::into_cluster(cube_points);
        self.set_nodes(cluster);

        
    }

    pub fn divide_into_spheres(&mut self) {
        let cube_points = self.cube_points();
        let cluster = OctreeSphere::into_cluster(cube_points);
        self.set_nodes(cluster);
    }

    pub fn node(&mut self, i: usize) -> &mut OctreeNode {
        match self {
            OctreeNode::Cube(cube) => {
                &mut cube.nodes.as_mut().unwrap()[i]
            }
            OctreeNode::Sphere(sphere) => {
                &mut  sphere.nodes.as_mut().unwrap()[i]
            }
        }
    }

    pub fn cube_points(&self) -> [[f32; 3]; 8] {
        match self {
            OctreeNode::Cube(cube) => {
                cube.cube_points.clone()
            }
            OctreeNode::Sphere(sphere) => {
                sphere.cube_points.clone()
            }
        }
    }

    pub fn nodes_mut(&mut self) -> Option<&mut std::boxed::Box<[OctreeNode; 8]>> {
        match self {
            OctreeNode::Cube(cube) => {
                cube.nodes.as_mut()
            }
            OctreeNode::Sphere(sphere) => {
                sphere.nodes.as_mut()
            }
        }
    }

    pub fn nodes_ref(&self) -> Option<&std::boxed::Box<[OctreeNode; 8]>> {
        match self {
            OctreeNode::Cube(cube) => {
                cube.nodes.as_ref()
            }
            OctreeNode::Sphere(sphere) => {
                sphere.nodes.as_ref()
            }
        }
    }

    pub fn set_nodes(&mut self, nodes: [OctreeNode; 8]) {
        match self {
            OctreeNode::Cube(cube) => {
                cube.nodes = Some(Box::new(nodes));
            }
            OctreeNode::Sphere(sphere) => {
                sphere.nodes = Some(Box::new(nodes));
            }
        }
    }

    pub fn add_data_point(&mut self, data_point: [f32; 3]) {
        match self {
            OctreeNode::Cube(cube) => {
                //cube.nodes = Some(Box::new(nodes));
            }
            OctreeNode::Sphere(sphere) => {
                sphere.data_points.push(data_point);
            }
        }
    }

    pub fn is_inside(&self, point: &[f32; 3]) -> bool {
        match self {
            OctreeNode::Cube(cube) => {
                false
            }
            OctreeNode::Sphere(sphere) => {
                sphere.is_inside(point)
                //false
            }
        }
    }

    pub fn import(&mut self, data_points: &[[f32; 3]]) {
        let nodes = self.nodes_mut();
        if let Some(nodes) = nodes {
            for node in nodes.iter_mut() {
                node.import(data_points);
            }
        } else {
            for data_point in data_points {
                if self.is_inside(data_point) {
                    self.add_data_point(*data_point);
                }
            }
        }

        // match self {
        //     OctreeNode::Cube(cube) => {
        //         false
        //     }
        //     OctreeNode::Sphere(sphere) => {
        //         sphere.imp
        //     }
        // }
    }

    pub fn export(&self, output: &mut Vec<[f32; 3]>) {
        match self {
            OctreeNode::Cube(cube) => {
                if let Some(nodes) = cube.nodes.as_ref() {
                    for node in nodes.iter() {
                        node.export(output);
                    }
                }
            }
            OctreeNode::Sphere(sphere) => {
                output.extend(sphere.data_points.clone());
                if let Some(nodes) = sphere.nodes.as_ref() {
                    for node in nodes.iter() {
                        node.export(output);
                    }
                }
            }
        }
    }
}

impl OctreeSphere {
    pub fn divide(&mut self) {
        self.nodes = Some(Box::new(Self::into_cluster(self.cube_points)));
    }

    pub fn into_cluster(cube: [[f32; 3]; 8]) -> [OctreeNode; 8] {
        let cluster = octree_divide_into_cube(&cube);
        Self::from_cluster(cluster)
    }

    pub fn from_cluster(points: [[[f32; 3]; 8]; 8]) -> [OctreeNode; 8] {
        let size = (points[0][1][1] - points[0][0][1]).abs() / 3.0;

        let [middle0_x, middle0_y, middle0_z] = cube_middle(&points[0]);
        let [middle1_x, middle1_y, middle1_z] = cube_middle(&points[1]);
        let [middle2_x, middle2_y, middle2_z] = cube_middle(&points[2]);
        let [middle3_x, middle3_y, middle3_z] = cube_middle(&points[3]);

        let [middle4_x, middle4_y, middle4_z] = cube_middle(&points[4]);
        let [middle5_x, middle5_y, middle5_z] = cube_middle(&points[5]);
        let [middle6_x, middle6_y, middle6_z] = cube_middle(&points[6]);
        let [middle7_x, middle7_y, middle7_z] = cube_middle(&points[7]);

        [
            OctreeNode::Sphere(OctreeSphere { cube_points: points[0], middle: [middle0_x, middle0_y, middle0_z], radius: size, sphere_points: gen_sphere(middle0_x, middle0_y, middle0_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[1], middle: [middle1_x, middle1_y, middle1_z], radius: size, sphere_points: gen_sphere(middle1_x, middle1_y, middle1_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[2], middle: [middle2_x, middle2_y, middle2_z], radius: size, sphere_points: gen_sphere(middle2_x, middle2_y, middle2_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[3], middle: [middle3_x, middle3_y, middle3_z], radius: size, sphere_points: gen_sphere(middle3_x, middle3_y, middle3_z, size), nodes: None, data_points: Vec::new() }),

            OctreeNode::Sphere(OctreeSphere { cube_points: points[4], middle: [middle4_x, middle4_y, middle4_z], radius: size, sphere_points: gen_sphere(middle4_x, middle4_y, middle4_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[5], middle: [middle5_x, middle5_y, middle5_z], radius: size, sphere_points: gen_sphere(middle5_x, middle5_y, middle5_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[6], middle: [middle6_x, middle6_y, middle6_z], radius: size, sphere_points: gen_sphere(middle6_x, middle6_y, middle6_z, size), nodes: None, data_points: Vec::new() }),
            OctreeNode::Sphere(OctreeSphere { cube_points: points[7], middle: [middle7_x, middle7_y, middle7_z], radius: size, sphere_points: gen_sphere(middle7_x, middle7_y, middle7_z, size), nodes: None, data_points: Vec::new() }),
        ]
    }

    pub fn is_inside(&self, point: &[f32; 3]) -> bool {
        // let data_point = nalgebra::Point3::new(point[0], point[1], point[2]);
        // let sphere_point = nalgebra::Point3::new(self.middle[0], self.middle[1], self.middle[2]);
        // let d = nalgebra::distance(&sphere_point, &data_point);
        // d <= self.radius
        point[0] > self.middle[0] - self.radius && 
        point[0] < self.middle[0] + self.radius &&

        point[1] > self.middle[1] - self.radius && 
        point[1] < self.middle[1] + self.radius &&

        point[2] > self.middle[2] - self.radius && 
        point[2] < self.middle[2] + self.radius
    }
}

impl OctreeCube {
    pub fn divide(&mut self) {
        self.nodes = Some(Box::new(Self::into_cluster(self.cube_points)));
    }

    pub fn into_cluster(cube: [[f32; 3]; 8]) -> [OctreeNode; 8] {
        let cluster = octree_divide_into_cube(&cube);
        Self::from_cluster(cluster)
    }

    pub fn from_cluster(points: [[[f32; 3]; 8]; 8]) -> [OctreeNode; 8] {
        [
            OctreeNode::Cube(OctreeCube { cube_points: points[0], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[1], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[2], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[3], nodes: None }),

            OctreeNode::Cube(OctreeCube { cube_points: points[4], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[5], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[6], nodes: None }),
            OctreeNode::Cube(OctreeCube { cube_points: points[7], nodes: None }),
        ]
    }
}

fn cube_middle(cube: &[[f32; 3]; 8]) -> [f32; 3] {
    let [start_x, start_y, start_z] = cube[0];
    let width_x = cube[3][0];

    let size = (width_x - start_x) / 2.0;
    let middle_x = start_x + size;
    let middle_y = start_y + size;
    let middle_z = start_z + size;

    [middle_x, middle_y, middle_z]
}

fn gen_cube(x: f32, y: f32, z: f32, size: f32) -> [[f32; 3]; 8] {
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

fn octree_divide_into_cube(octree_cube: &[[f32; 3]; 8]) -> [[[f32; 3]; 8]; 8] {
    let [start_x, start_y, start_z] = octree_cube[0];
    //let [middle_x, middle_y, middle_z] = cube_middle(octree_cube);
    let [start_x, start_y, start_z] = octree_cube[0];
    let width_x = octree_cube[3][0];

    let size = (width_x - start_x) / 2.0;
    let middle_x = start_x + size;
    let middle_y = start_y + size;
    let middle_z = start_z + size;

    [
        gen_cube(start_x, start_y, start_z, size),
        gen_cube(start_x, middle_y, start_z, size),
        gen_cube(middle_x, middle_y, start_z, size),
        gen_cube(middle_x, start_y, start_z, size),

        gen_cube(start_x, start_y, middle_z, size),
        gen_cube(start_x, middle_y, middle_z, size),
        gen_cube(middle_x, middle_y, middle_z, size),
        gen_cube(middle_x, start_y, middle_z, size),

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



// struct OctreeSphere {}

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

    let (mut points, header) = read_las("/home/hey/Downloads/2743_1234.las");
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

    let mut tree = Octree::new(size);
    println!("TREE {:#?}", tree);
    let lines = gen_debug_lines(&tree);

    println!("importing...");
    
    
    transform_move(&mut points, &bounds);
    tree.import(&points);

    println!("exporting...");
    let modified_points = tree.export();

    //let mut modified_points = octotree(&points, &bounds);
    drop(points);

    

    let mesh: Mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, modified_points);

    
    
    //let size = [bounds.max.x, bounds.max.y , bounds.max.z].into_iter().max().unwrap() as f32;

    //max!

    
    //let sphere = gen_sphere(-200.0, 350.0, 100.0, 20.0);
    // /println!("SPHERE {:#?}", sphere);
    let mesh = meshes.add(mesh);
    let lines = meshes.add(lines);
    //let sphere = meshes.add(sphere);

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
            material: debug_material.clone(),
            //material: debug_material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            // .with_rotation(Quat::from_rotation_x(45.0)),
            ..default()
        },
        Shape,
    ));

    // commands.spawn((
    //     MaterialMeshBundle {
    //         mesh: sphere,
    //         material: debug_material,
    //         //material: debug_material,
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //         // .with_rotation(Quat::from_rotation_x(45.0)),
    //         ..default()
    //     },
    //     Shape,
    // ));


    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         shadows_enabled: true,
    //         intensity: 10_000_000.,
    //         range: 100.0,
    //         shadow_depth_bias: 0.2,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(8.0, 16.0, 8.0),
    //     ..default()
    // });

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
    

    let mut queue = VecDeque::<&OctreeNode>::new();

    queue.push_back(&tree.root);
    // if let Some(nodes) = &tree.root.nodes {
    //     //queue.push_back(&nodes[0]) ;
    //     for node in nodes.iter().skip(1) {
    //         queue.push_back(node);
    //     }
    // }
    

    let mut i: u32 = 0;
    loop {
        let Some(node) = queue.pop_front() else {
            break;
        };

        match node {
            OctreeNode::Cube(cube) => {
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
            OctreeNode::Sphere(sphere) => {

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

fn gen_sphere(origin_x: f32, origin_y: f32, origin_z: f32, radius: f32) -> [[f32; 3]; 12] {
    // let origin_x = origin_x + radius / 2.0;
    // let origin_y = origin_y + radius / 2.0;
    // let origin_z = origin_z + radius / 2.0;
    // let mut pos: Vec<[f32; 3]> = vec![[-100.0, 0.0, 0.0], [-100.0, 0.0, 100.0]];
    // let mut indices: Vec<u32> = vec![0, 1];
    

    let angle_step = 360.0 / 5.0;
    let angle_ratio = PI * (2.0 / 360.0);
    let diameter = radius * 2.0; 
    let diameter_ratio = diameter / 3.0;
    let hat_height = diameter_ratio;
    let bridge_height = diameter_ratio * 2.0;

    let origin_y = origin_y + diameter_ratio;

    let edge0_angle =  angle_ratio * (angle_step * 1.0);
    let edge0_angle_sin = edge0_angle.sin();
    let edge0_angle_cos = edge0_angle.cos();

    let edge1_angle =  angle_ratio * (angle_step * 2.0);
    let edge1_angle_sin = edge1_angle.sin();
    let edge1_angle_cos = edge1_angle.cos();

    let edge2_angle =  angle_ratio * (angle_step * 3.0);
    let edge2_angle_sin = edge2_angle.sin();
    let edge2_angle_cos = edge2_angle.cos();

    let edge3_angle =  angle_ratio * (angle_step * 4.0);
    let edge3_angle_sin = edge3_angle.sin();
    let edge3_angle_cos = edge3_angle.cos();

    let edge4_angle =  angle_ratio * (angle_step * 5.0);
    let edge4_angle_sin = edge4_angle.sin();
    let edge4_angle_cos = edge4_angle.cos();

    let edge5_angle =  angle_ratio * (angle_step * 1.0 + 36.0);
    let edge5_angle_sin = edge5_angle.sin();
    let edge5_angle_cos = edge5_angle.cos();

    let edge6_angle =  angle_ratio * (angle_step * 2.0 + 36.0);
    let edge6_angle_sin = edge6_angle.sin();
    let edge6_angle_cos = edge6_angle.cos();

    let edge7_angle =  angle_ratio * (angle_step * 3.0 + 36.0);
    let edge7_angle_sin = edge7_angle.sin();
    let edge7_angle_cos = edge7_angle.cos();

    let edge8_angle =  angle_ratio * (angle_step * 4.0 + 36.0);
    let edge8_angle_sin = edge8_angle.sin();
    let edge8_angle_cos = edge8_angle.cos();

    let edge9_angle =  angle_ratio * (angle_step * 5.0 + 36.0);
    let edge9_angle_sin = edge9_angle.sin();
    let edge9_angle_cos = edge9_angle.cos();

    let pos: [[f32; 3]; 12] = [
        [origin_x + radius * edge0_angle_cos - radius * edge0_angle_sin, origin_y, origin_z + radius * edge0_angle_sin + radius * edge0_angle_cos],
        [origin_x + radius * edge1_angle_cos - radius * edge1_angle_sin, origin_y, origin_z + radius * edge1_angle_sin + radius * edge1_angle_cos],
        [origin_x + radius * edge2_angle_cos - radius * edge2_angle_sin, origin_y, origin_z + radius * edge2_angle_sin + radius * edge2_angle_cos],
        [origin_x + radius * edge3_angle_cos - radius * edge3_angle_sin, origin_y, origin_z + radius * edge3_angle_sin + radius * edge3_angle_cos],
        [origin_x + radius * edge4_angle_cos - radius * edge4_angle_sin, origin_y, origin_z + radius * edge4_angle_sin + radius * edge4_angle_cos],

        [origin_x, origin_y + hat_height, origin_z],

        [origin_x + radius * edge5_angle_cos - radius * edge5_angle_sin, origin_y - bridge_height, origin_z + radius * edge5_angle_sin + radius * edge5_angle_cos],
        [origin_x + radius * edge6_angle_cos - radius * edge6_angle_sin, origin_y - bridge_height, origin_z + radius * edge6_angle_sin + radius * edge6_angle_cos],
        [origin_x + radius * edge7_angle_cos - radius * edge7_angle_sin, origin_y - bridge_height, origin_z + radius * edge7_angle_sin + radius * edge7_angle_cos],
        [origin_x + radius * edge8_angle_cos - radius * edge8_angle_sin, origin_y - bridge_height, origin_z + radius * edge8_angle_sin + radius * edge8_angle_cos],
        [origin_x + radius * edge9_angle_cos - radius * edge9_angle_sin, origin_y - bridge_height, origin_z + radius * edge9_angle_sin + radius * edge9_angle_cos],

        [origin_x, origin_y + hat_height * -3.0, origin_z],
    ];

    
    
    //let x = &mut pos[1][0];
    //let y = &mut pos[1][1];

    
    //let [origin_x, origin_y, origin_z] = [-100.0, 0.0, 0.0];

    //pos.extend([[origin_x, origin_y, origin_z]]);

    
    // let diameter = radius * 2.0;
    // let edge_count: u32 = 5;
    // let edge_count_sqr = (edge_count as f32).sqrt();
    // //let height = r * (1.0 - 1.0/(edge_count as f32).sqrt());
    // // let hat_height = (r / 3.0) + /2.0 * (1.0 - 1.0/edge_count_sqr);
    // // let bridge_height = r/2.0 + edge_count_sqr;
    // let hat_height = diameter / 3.0;
    // let bridge_height = (diameter / 3.0) * 2.0;

    // let mut edge_i: u32 = 0;

    // let mut add_half = |height_mul: f32, deg: f32, origin: [f32; 3]| {
    //     let [origin_x, origin_y, origin_z] = origin;
    //     let new_edge_count = edge_i + edge_count;

    //     for i in edge_i + 1..=new_edge_count {
    //         let d = (PI * (2.0 / 360.0)) * (angle_step * i as f32 + deg);
    //         let d_cos = d.cos();
    //         let d_sin = d.sin();
    
    //         let mut x = radius;
    //         let mut z = radius;
            
    //         // x = origin_x + (x - origin_x) * r_cos - (z - origin_z) * r_sin;
    //         // z = origin_z + (z - origin_x) * r_sin + (z - origin_z) * r_cos;
    
    //         x = origin_x + x * d_cos - x * d_sin;
    //         z = origin_z + z * d_sin + z * d_cos;
    
    //         pos.extend([[x, origin_y, z]]);
    //     }
    
    //     for i in edge_i..new_edge_count - 1 {
    //         indices.extend([i, i + 1])
    //     }
    
    //     indices.extend([edge_i, new_edge_count - 1]);
    
        
    //     pos.extend([[origin_x, origin_y + (hat_height * height_mul), origin_z]]);
    
    //     for i in edge_i..=new_edge_count - 1 {
    //         indices.extend([i, new_edge_count])
    //     }

    //     edge_i += edge_count + 1;
    // };

    // add_half(1.0, 0.0, [origin_x, origin_y, origin_z]);
    // add_half(-1.0, 36.0, [origin_x, origin_y - (bridge_height ), origin_z]);

    // for i in 0..edge_count {
    //     // let first_half = pos[i as usize];
    //     // let second_half = pos[(i + edge_count) as usize + 1];
    //     indices.extend([i, i + edge_count, i, i + edge_count - 1]);
    // }

    // indices.extend([0, edge_count * 2, 0,  edge_count * 2 - 1, 1, edge_count * 2]);
    

    // let lines = Mesh::new(
    //     bevy::render::mesh::PrimitiveTopology::LineList,
    //     RenderAssetUsages::default(),
    // )
    // .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
    // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos);

    pos
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
