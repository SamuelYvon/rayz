use raylib::math::Vector3;
use std::cmp::Ordering;
use std::collections::HashMap;

const MAX_MARCHING_STEPS: usize = 100;
const EPSILON: f32 = 0.0001;

pub type SdfId = usize;

pub trait Sdf: Sync {
    fn id(&self) -> SdfId;

    fn sdf(&self, v: Vector3) -> f32;

    fn dist(&self, v: Vector3) -> (f32, SdfId) {
        (self.sdf(v), self.id())
    }

    /// Return the surface normal of the SDF (as a normalized vector)
    fn surface_normal(&self, point: Vector3) -> Vector3 {
        let (x, y, z) = (point.x, point.y, point.z);

        const EPSILON: f32 = 0.0005;

        let dx =
            self.sdf(Vector3::new(x + EPSILON, y, z)) - self.sdf(Vector3::new(x - EPSILON, y, z));

        let dy =
            self.sdf(Vector3::new(x, y + EPSILON, z)) - self.sdf(Vector3::new(x, y - EPSILON, z));

        let dz =
            self.sdf(Vector3::new(x, y, z + EPSILON)) - self.sdf(Vector3::new(x, y, z - EPSILON));

        Vector3::new(dx, dy, dz).normalized()
    }
}

pub struct Sphere {
    pub id: SdfId,
    pub center: Vector3,
    pub radius: f32,
}

impl Sdf for Sphere {
    fn id(&self) -> SdfId {
        self.id
    }

    fn sdf(&self, v: Vector3) -> f32 {
        (v - self.center).length() - self.radius
    }
}

pub struct Cube {
    pub id: SdfId,
    pub center: Vector3,
    pub length: f32,
}

fn absolute(vector3: Vector3) -> Vector3 {
    Vector3::new(vector3.x.abs(), vector3.y.abs(), vector3.z.abs())
}

impl Sdf for Cube {
    fn id(&self) -> SdfId {
        self.id
    }

    fn sdf(&self, p: Vector3) -> f32 {
        // https://iquilezles.org/articles/distfunctions/
        // float sdBox( vec3 p, vec3 b )
        // {
        //    vec3 q = abs(p) - b;
        //    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
        // }
        let q = absolute(p - self.center) - self.length;
        let zero = Vector3::default();

        q.max(zero).length() + q.y.max(q.z).max(q.x).min(0.0)
    }
}

pub struct Scene {
    objects: HashMap<SdfId, Box<dyn Sdf>>,
}

impl Scene {
    pub fn new(objs: Vec<Box<dyn Sdf>>) -> Self {
        let mut objects = HashMap::new();

        for o in objs {
            objects.insert(o.id(), o);
        }

        Self { objects }
    }

    pub fn get_object(&self, id: SdfId) -> &dyn Sdf {
        self.objects.get(&id).unwrap().as_ref()
    }

    /// Ray-march until something is reached. Returns the point where the ray has it
    /// as well as the id of the object hit.
    pub fn ray_march(&self, pos: Vector3, ray: Vector3) -> Option<(Vector3, SdfId)> {
        let mut depth = 0.00;

        for _ in 0..MAX_MARCHING_STEPS {
            let point = pos + ray * depth;

            let (dist, obj) = match self
                .objects
                .values()
                .map(|obj| obj.dist(point))
                // Union of the objects
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Less))
            {
                Some(v) => v,
                None => return None,
            };

            if dist <= EPSILON {
                return Some((point, obj));
            }

            depth += dist
        }

        None
    }
}
