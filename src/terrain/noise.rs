use std::sync::LazyLock;

// Permutation table for Perlin noise
static PERMUTATION: LazyLock<[u8; 512]> = LazyLock::new(|| {
    let mut p = [0u8; 512];
    let base = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140,
        36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120,
        234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33,
        88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175, 74, 165, 71,
        134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133,
        230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161,
        1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130,
        116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250,
        124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227,
        47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44,
        154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19,
        98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228,
        251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235,
        249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176,
        115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29,
        24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];

    p[..256].copy_from_slice(&base);
    p[256..(256 + 256)].copy_from_slice(&base);
    p
});

pub struct PerlinNoise {
    /// Seed-shuffled permutation; distinct seeds give distinct worlds.
    perm: [u8; 512],
}

impl PerlinNoise {
    pub fn new(seed: u32) -> Self {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        let mut base: [u8; 256] = PERMUTATION[..256].try_into().unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed as u64);
        base.shuffle(&mut rng);

        let mut perm = [0u8; 512];
        perm[..256].copy_from_slice(&base);
        perm[256..].copy_from_slice(&base);
        Self { perm }
    }

    /// 3D Perlin noise. Sampling a 2D heightmap on a cylinder
    /// (x -> cos/sin pair, y stays) gives a seamless horizontal wrap.
    pub fn noise_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = (x.floor() as i32) & 255;
        let yi = (y.floor() as i32) & 255;
        let zi = (z.floor() as i32) & 255;

        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();

        let u = fade(xf);
        let v = fade(yf);
        let w = fade(zf);

        // Classic double-permutation corner hashing: perm values are u8, and
        // coordinate offsets stay in [0, 256), so & 255 is a safe wrap.
        let perm = &self.perm;
        let hash = |i: i32, j: i32, k: i32| -> usize {
            let a = (perm[(i & 255) as usize] as i32 + j) & 255;
            let b = (perm[a as usize] as i32 + k) & 255;
            perm[b as usize] as usize
        };

        let aaa = hash(xi, yi, zi);
        let aba = hash(xi, yi + 1, zi);
        let aab = hash(xi, yi, zi + 1);
        let abb = hash(xi, yi + 1, zi + 1);
        let baa = hash(xi + 1, yi, zi);
        let bba = hash(xi + 1, yi + 1, zi);
        let bab = hash(xi + 1, yi, zi + 1);
        let bbb = hash(xi + 1, yi + 1, zi + 1);

        let x1 = lerp(grad_3d(aaa, xf, yf, zf), grad_3d(baa, xf - 1.0, yf, zf), u);
        let x2 = lerp(
            grad_3d(aba, xf, yf - 1.0, zf),
            grad_3d(bba, xf - 1.0, yf - 1.0, zf),
            u,
        );
        let x3 = lerp(
            grad_3d(aab, xf, yf, zf - 1.0),
            grad_3d(bab, xf - 1.0, yf, zf - 1.0),
            u,
        );
        let x4 = lerp(
            grad_3d(abb, xf, yf - 1.0, zf - 1.0),
            grad_3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0),
            u,
        );

        let y1 = lerp(x1, x2, v);
        let y2 = lerp(x3, x4, v);

        lerp(y1, y2, w)
    }

    pub fn octave_noise_3d(
        &self,
        x: f64,
        y: f64,
        z: f64,
        octaves: i32,
        persistence: f64,
        scale: f64,
    ) -> f64 {
        let mut total = 0.0;
        let mut frequency = scale;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.noise_3d(x * frequency, y * frequency, z * frequency)
                * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        total / max_value
    }
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

fn grad_3d(hash: usize, x: f64, y: f64, z: f64) -> f64 {
    // Classic 12-gradient Perlin scheme: pick direction by hash bits,
    // then the dominant two axes carry the gradient.
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    let sign_u = if h & 1 == 0 { u } else { -u };
    let sign_v = if h & 2 == 0 { v } else { -v };
    sign_u + sign_v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_3d_is_bounded_and_varies() {
        let n = PerlinNoise::new(1);
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for i in 0..200 {
            let v = n.noise_3d(i as f64 * 0.37, i as f64 * 0.11, i as f64 * 0.23);
            min = min.min(v);
            max = max.max(v);
            assert!(v.abs() <= 1.5);
        }
        assert!(max - min > 0.5, "noise should vary, got [{min}, {max}]");
    }

    #[test]
    fn noise_3d_is_continuous() {
        let n = PerlinNoise::new(2);
        let a = n.noise_3d(1.0, 2.0, 3.0);
        let b = n.noise_3d(1.001, 2.0, 3.0);
        assert!((a - b).abs() < 0.01);
    }
}
