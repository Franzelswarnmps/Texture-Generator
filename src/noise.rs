use std::{collections::HashMap, vec};

use crate::{sprite_gen::SpriteGen};
use noise::{utils::*, *};

use rand::{
    prelude::{SliceRandom, ThreadRng},
    Rng,
};

pub fn noise_fill(sprite: &mut SpriteGen) {
    let mut rng = rand::thread_rng();

    let perlin = Perlin::new().set_seed(rng.gen());
    let terrace = Terrace::new(&perlin)
        .add_control_point(-1.0)
        .add_control_point(-0.5)
        .add_control_point(0.1)
        .add_control_point(1.0);
    let map = PlaneMapBuilder::new(&terrace)
        .set_size(sprite.dimensions.0, sprite.dimensions.1)
        .build();

    let noise_plateau = noise_plateau(&mut rng, &sprite.char_color);
    for index in 0..sprite.char_texture.pixels.len() {
        let (x,y) = sprite.char_texture.xy_from_index(index);
        let possible_letters = get_noise_plateau_level(&noise_plateau,map.get_value(x, y));
        sprite.char_texture.set(x, y, *possible_letters.choose(&mut rng).unwrap());
    }
}

fn noise_plateau(rng: &mut ThreadRng, letter_colors: &HashMap<char, [u8; 4]>) -> Vec<(f64,Vec<char>)> {
    let letters: Vec<char> = letter_colors.keys().map(|s| s.to_owned()).collect();

    let mut diff_pairs = vec![];
    for (k1,v1) in letter_colors.iter() {
        for (k2,v2) in letter_colors.iter() {
            if *k1 != *k2 {
                let sum1: i32 = v1.iter().map(|i| *i as i32).sum();
                let sum2: i32 = v2.iter().map(|i| *i as i32).sum();
                diff_pairs.push((*k1,*k2,(sum1 - sum2).abs() as u32));
            }
        }
    }
    // let total_diff: u32 = diff_pairs.iter().map(|(_,_,n)| n).sum();
    // let avg_diff = total_diff / letter_colors.keys().len() as u32;

    let avg_diff = 100; // TODO make config

    let mut letter_groups: HashMap<char,Vec<char>> = letters.iter().map(|l| (l.to_owned(),vec![l.to_owned()])).collect();
    for (letter1,letter2,diff) in diff_pairs {
        if diff < avg_diff {
            letter_groups.entry(letter1)
                .or_insert_with(Vec::new)
                .push(letter2);
        }
    }

    let mut layers: Vec<f64> = (0..letter_groups.len() - 1).map(|_| rng.gen_range(0.0..1.0)).collect();
    layers.push(1.0); // plateau selected between previous-next pairs. 0.0 is set as
    // the first previous. 1.0 neded for full coverage.
    layers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    layers.into_iter().zip(letter_groups.into_values()).collect()
}

fn get_noise_plateau_level(levels: &[(f64,Vec<char>)], noise: f64) -> &[char] {
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