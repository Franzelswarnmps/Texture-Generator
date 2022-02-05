use std::{collections::HashMap, vec};

use crate::sprite_gen::SpriteGen;
use noise::{utils::*, *};

use rand::{
    prelude::{SliceRandom, ThreadRng},
    Rng,
};
/*
random number of generators. repeats fine, random seeds
random number of modifiers, applied to random generators, repeats allowed
random number of combiners, applied to random generators, nesting allowed
blend everything
while generating values, track min/max
go over values at end and remap to [0,1] (val - min) / (max - min)
*/
struct NoiseSettings {
    generator_chance: f32,
    modify_chance: f32,
    combine_chance: f32,
}

impl NoiseSettings {
    pub fn random(rng: &mut ThreadRng) -> Self {
        Self {
            generator_chance: rng.gen_range(0.1..0.5),
            modify_chance: rng.gen_range(0.2..0.9),
            combine_chance: rng.gen_range(0.2..0.9),
        }
    }
}

/*
for each generator,
chance to modify -> apply modify -> repeat until did not hit
combine chance try once -> if true, select one of the existing generators to combine

if more than 1 left, go over all with modify chance one more time
blend

while generating values, track min/max
go over values at end and remap to [0,1]

double slope = 1.0 * (output_end - output_start) / (input_end - input_start)
output = output_start + round(slope * (input - input_start))
*/

// https://stackoverflow.com/questions/5731863/mapping-a-numeric-range-onto-another
fn map_range(scale: f64, dest_start: f64, value: f64) -> f64 {
    dest_start + scale * value
}
fn map_range_scale(source_range: (f64,f64), dest_range: (f64,f64)) -> f64 {
    let source_len = source_range.1 - source_range.0;
    let dest_len = dest_range.1 - dest_range.0;
    dest_len / source_len
}

#[derive(Clone,Copy)]
struct GeneratedNoiseSettings {
    size: (usize,usize),
    x_bounds: (f64,f64),
    y_bounds: (f64,f64),
}

struct GeneratedNoise {
    pub noise: NoiseMap,
    settings: GeneratedNoiseSettings,
}

impl GeneratedNoise {
    fn from_noise(source: &dyn NoiseFn<[f64; 3]>, settings: GeneratedNoiseSettings) -> Self {
        Self {
            noise: PlaneMapBuilder::new(source)
                .set_size(settings.size.0, settings.size.1)
                .set_x_bounds(settings.x_bounds.0, settings.x_bounds.1)
                .set_y_bounds(settings.y_bounds.0, settings.y_bounds.1)
                .build(),
            settings,
        }
    }

    fn rescale(&mut self, min: f64, max: f64) {
        let mut current_min = f64::INFINITY;
        let mut current_max = f64::NEG_INFINITY;
        //determine current range
        for y in 0..self.noise.size().1 {
            for x in 0..self.noise.size().0 {
                let current_value = self.noise.get_value(x, y);
                if current_min > current_value {
                    current_min = current_value;
                }
                if current_max < current_value {
                    current_max = current_value;
                }
            }
        }
        let dest_range = (min,max);
        let range_scale = map_range_scale((current_min,current_max), dest_range);
        // map to new range
        for y in 0..self.noise.size().1 {
            for x in 0..self.noise.size().0 {
                let mapped_value = map_range(range_scale, dest_range.0, self.noise.get_value(x, y));
                self.noise.set_value(x, y, mapped_value);
            }
        }
    }
}

impl NoiseFn<[f64; 3]> for GeneratedNoise {
    fn get(&self, point: [f64; 3]) -> f64 {
        // map f64 within self.bounds to usize within self.size
        let x_len = self.settings.x_bounds.1 - self.settings.x_bounds.0;
        let x_step = x_len / self.settings.size.0 as f64;
        let x = (point[0] / x_step) as usize;

        let y_len = self.settings.y_bounds.1 - self.settings.y_bounds.0;
        let y_step = y_len / self.settings.size.1 as f64;
        let y = (point[1] / y_step) as usize;

        self.noise.get_value(x, y)
    }
}

fn random_generator(rng: &mut ThreadRng, settings: GeneratedNoiseSettings) -> GeneratedNoise {
    match rng.gen_range(0..=8) {
        0 => GeneratedNoise::from_noise(&Perlin::new().set_seed(rng.gen()), settings),
        1 => GeneratedNoise::from_noise(&Worley::new().set_seed(rng.gen()), settings),
        2 => GeneratedNoise::from_noise(&Fbm::new().set_seed(rng.gen()), settings),
        3 => GeneratedNoise::from_noise(&Billow::new().set_seed(rng.gen()), settings),
        4 => GeneratedNoise::from_noise(&Checkerboard::new(rng.gen_range(1..10)), settings),
        5 => GeneratedNoise::from_noise(&Cylinders::new().set_frequency(rng.gen_range(0.1..10.0)), settings),
        6 => GeneratedNoise::from_noise(&OpenSimplex::new().set_seed(rng.gen()), settings),
        7 => GeneratedNoise::from_noise(&SuperSimplex::new().set_seed(rng.gen()), settings),
        8 => GeneratedNoise::from_noise(&Value::new().set_seed(rng.gen()), settings),
        _ => panic!("random generator range wrong"),
    }
}

fn random_modifier(rng: &mut ThreadRng, source: &dyn NoiseFn<[f64; 3]>, settings: GeneratedNoiseSettings) 
-> GeneratedNoise {
    match rng.gen_range(0..=4) {
        0 => GeneratedNoise::from_noise(&Abs::new(source), settings),
        1 => GeneratedNoise::from_noise(&Clamp::new(source).set_bounds(rng.gen_range(-1.0..0.0), rng.gen_range(0.0..1.0),), settings),
        2 => GeneratedNoise::from_noise(&Exponent::new(source).set_exponent(rng.gen_range(0.1..2.0)), settings),
        3 => GeneratedNoise::from_noise(&Negate::new(source), settings),
        4 => GeneratedNoise::from_noise(&ScaleBias::new(source).set_scale(rng.gen_range(0.1..3.0)).set_bias(rng.gen_range(0.0..1.0)), settings),
        _ => panic!("random modifier range wrong"),
    }
}

fn random_combiner(
    rng: &mut ThreadRng,
    source1: &dyn NoiseFn<[f64; 3]>,
    source2: &dyn NoiseFn<[f64; 3]>, 
    settings: GeneratedNoiseSettings) 
    -> GeneratedNoise {
    match rng.gen_range(0..=4) {
        0 => GeneratedNoise::from_noise(&Add::new(source1, source2), settings),
        1 => GeneratedNoise::from_noise(&Max::new(source1, source2), settings),
        2 => GeneratedNoise::from_noise(&Min::new(source1, source2), settings),
        3 => GeneratedNoise::from_noise(&Multiply::new(source1, source2), settings),
        4 => GeneratedNoise::from_noise(&Power::new(source1, source2), settings),
        _ => panic!("random modifier range wrong"),
    }
}

fn random_noise(sprite: &mut SpriteGen) -> NoiseMap {
    let mut rng = rand::thread_rng();
    let settings = NoiseSettings::random(&mut rng);

    let mut last_layer: Vec<GeneratedNoise> = vec![];
    let layer_settings = GeneratedNoiseSettings {
        size: sprite.dimensions,
        x_bounds: (-3.,3.),
        y_bounds: (-3.,3.),
    };

    // generators
    last_layer.push(random_generator(&mut rng, layer_settings)); // need at least 1
    while rng.gen_range(0.0..1.0) < settings.generator_chance {
        last_layer.push(random_generator(&mut rng, layer_settings));
    }

    // main layer loop
    while last_layer.len() > 1 {
        let mut next_layer= vec![];

        // combine
        let mut used_indices = vec![false; last_layer.len()];
        while next_layer.len() < last_layer.len() && rng.gen_range(0.0..1.0) < settings.combine_chance {
            let source1_index = rng.gen_range(0..last_layer.len());
            let source2_index = rng.gen_range(0..last_layer.len());
            used_indices[source1_index] = true;
            used_indices[source2_index] = true;
            next_layer.push(random_combiner(&mut rng, &last_layer[source1_index], &last_layer[source2_index], layer_settings));
        }

        // any not used, chance for modify
        for unused_index in (0..last_layer.len()).zip(used_indices.clone()).filter(|(_, i)| !*i).map(|(g, _)| g) {
            if rng.gen_range(0.0..1.0) < settings.modify_chance {
                used_indices[unused_index] = true;
                next_layer.push(random_modifier(&mut rng, &last_layer[unused_index], layer_settings));
            }
        }

        //carryover unused
        for unused in last_layer.into_iter().zip(used_indices).filter(|(_, i)| !*i).map(|(g, _)| g) {
            next_layer.push(unused);
        }
        last_layer = next_layer;
    }

    // get map
    let mut map = last_layer.into_iter().last().unwrap();
    map.rescale(0., 1.);
    map.noise
}

pub fn noise_fill(sprite: &mut SpriteGen) {
    let mut rng = rand::thread_rng();

    let map = random_noise(sprite);

    let noise_plateau = noise_plateau(&mut rng, &sprite.char_color);
    for index in 0..sprite.char_texture.pixels.len() {
        let (x, y) = sprite.char_texture.xy_from_index(index);
        let possible_letters = get_noise_plateau_level(&noise_plateau, map.get_value(x, y));
        sprite
            .char_texture
            .set(x, y, *possible_letters.choose(&mut rng).unwrap());
    }
}

fn noise_plateau(
    rng: &mut ThreadRng,
    letter_colors: &HashMap<char, [u8; 4]>,
) -> Vec<(f64, Vec<char>)> {
    let letters: Vec<char> = letter_colors.keys().map(|s| s.to_owned()).collect();

    let mut diff_pairs = vec![];
    for (k1, v1) in letter_colors.iter() {
        for (k2, v2) in letter_colors.iter() {
            let sum1: i32 = v1.iter().map(|i| *i as i32).sum();
            let sum2: i32 = v2.iter().map(|i| *i as i32).sum();
            diff_pairs.push((*k1, *k2, (sum1 - sum2).abs() as u32));
        }
    }
    // let total_diff: u32 = diff_pairs.iter().map(|(_,_,n)| n).sum();
    // let avg_diff = total_diff / letter_colors.keys().len() as u32;

    let avg_diff = 100; // TODO make config

    let mut letter_groups: HashMap<char, Vec<char>> = letters
        .iter()
        .map(|l| (l.to_owned(), vec![l.to_owned()]))
        .collect();
    for (letter1, letter2, diff) in diff_pairs {
        if diff < avg_diff {
            letter_groups
                .entry(letter1)
                .or_insert_with(Vec::new)
                .push(letter2);
        }
    }

    let mut layers: Vec<f64> = (0..letter_groups.len() - 1)
        .map(|_| rng.gen_range(0.0..1.0))
        .collect();
    layers.push(1.0); // plateau selected between previous-next pairs. 0.0 is set as
                      // the first previous. 1.0 neded for full coverage.
    layers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    layers
        .into_iter()
        .zip(letter_groups.into_values())
        .collect()
}

fn get_noise_plateau_level(levels: &[(f64, Vec<char>)], noise: f64) -> &[char] {
    let mut prev = 0.0;
    let mut result = &levels[0].1;
    for (threshold, letters) in levels {
        if noise >= prev && noise <= *threshold {
            result = letters;
            break;
        }
        prev = *threshold;
    }
    result
}
