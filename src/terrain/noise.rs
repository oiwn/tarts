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

    for i in 0..256 {
        p[i] = base[i];
        p[256 + i] = base[i];
    }
    p
});

pub struct PerlinNoise {
    seed: u32,
}

impl PerlinNoise {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    pub fn noise_2d(&self, x: f64, y: f64) -> f64 {
        // Find unit square containing point
        let xi = (x.floor() as i32) & 255;
        let yi = (y.floor() as i32) & 255;

        // Find relative position in square
        let xf = x - x.floor();
        let yf = y - y.floor();

        // Compute fade curves
        let u = fade(xf);
        let v = fade(yf);

        // Hash coordinates of square corners
        let aa = PERMUTATION
            [(PERMUTATION[xi as usize] as usize + yi as usize) % 512]
            as usize;
        let ab = PERMUTATION
            [(PERMUTATION[xi as usize] as usize + yi as usize + 1) % 512]
            as usize;
        let ba = PERMUTATION
            [(PERMUTATION[(xi + 1) as usize % 256] as usize + yi as usize) % 512]
            as usize;
        let bb = PERMUTATION[(PERMUTATION[(xi + 1) as usize % 256] as usize
            + yi as usize
            + 1)
            % 512] as usize;

        // Interpolate between gradients
        let x1 = lerp(grad_2d(aa, xf, yf), grad_2d(ba, xf - 1.0, yf), u);
        let x2 = lerp(
            grad_2d(ab, xf, yf - 1.0),
            grad_2d(bb, xf - 1.0, yf - 1.0),
            u,
        );

        lerp(x1, x2, v)
    }

    pub fn octave_noise_2d(
        &self,
        x: f64,
        y: f64,
        octaves: i32,
        persistence: f64,
        scale: f64,
    ) -> f64 {
        let mut total = 0.0;
        let mut frequency = scale;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.noise_2d(x * frequency, y * frequency) * amplitude;
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

fn grad_2d(hash: usize, x: f64, y: f64) -> f64 {
    let h = hash & 3;
    match h {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        _ => 0.0,
    }
}
