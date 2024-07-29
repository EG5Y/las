use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Octree {
    pub root: OctreeNode, 
}

#[derive(Debug, Clone)]
pub struct OctreeCube {
    pub cube_points: [[f32; 3]; 8],
    pub nodes: Option<Box<[OctreeNode; 8]>>,
}

#[derive(Debug, Clone)]
pub struct OctreeSphere {
    pub middle: [f32; 3],
    pub radius: f32,
    pub data_points: Vec<[f32; 3]>,
    pub cube_points: [[f32; 3]; 8],
    pub sphere_points: [[f32; 3]; 12],
    pub nodes: Option<Box<[OctreeNode; 8]>>,
}

#[derive(Debug, Clone)]
pub enum OctreeNode {
    Cube(OctreeCube),
    Sphere(OctreeSphere),
}

impl Octree {
    pub fn new(size: f32) -> Self {
        let root = gen_cube(0.0, 0.0, 0.0, size);

        
        let mut nodes = OctreeCube::into_cluster(root);
        for node in nodes.iter_mut() {
            node.divide_into_spheres();
            for node in node.nodes_mut().unwrap().iter_mut() {
                node.divide_into_spheres();
            }
        }

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

pub fn cube_middle(cube: &[[f32; 3]; 8]) -> [f32; 3] {
    let [start_x, start_y, start_z] = cube[0];
    let width_x = cube[3][0];

    let size = (width_x - start_x) / 2.0;
    let middle_x = start_x + size;
    let middle_y = start_y + size;
    let middle_z = start_z + size;

    [middle_x, middle_y, middle_z]
}

pub fn gen_cube(x: f32, y: f32, z: f32, size: f32) -> [[f32; 3]; 8] {
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

pub fn gen_sphere(origin_x: f32, origin_y: f32, origin_z: f32, radius: f32) -> [[f32; 3]; 12] {

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

    pos
}

pub fn octree_divide_into_cube(octree_cube: &[[f32; 3]; 8]) -> [[[f32; 3]; 8]; 8] {
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
