use std::collections::BTreeMap;

use crate::rule::Rule;

pub fn texture_to_png_base64(data: Vec<u8>, width: usize, height: usize) -> String {
    let mut output: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut output);
    if image::write_buffer_with_format(
        &mut cursor,
        &data,
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png
    ).is_err() {
        println!("failed to save image");
    };

    format!("{}{}","data:image/png;base64,",base64::encode(output))
}

pub fn serialize_config(rules: &Vec<Rule>, colors: &BTreeMap<char, [u8; 4]>) -> String {
    let mut rules_data: Vec<(String,String)> = vec![];
    for rule in rules {
        rules_data.push((rule.original_condition().to_string(), rule.original_action().to_string()));
    }
    let mut colors_data: Vec<(char, [u8; 4])> = vec![];
    for (letter,color) in colors.iter() {
        colors_data.push((*letter,*color));
    }
    serde_json::to_string(&(rules_data,colors_data)).unwrap()
}

pub fn deserialize_config(data: &str) -> Option<(Vec<Rule>,BTreeMap<char, [u8; 4]>)> {
    let (parsed_rules,parsed_colors): (Vec<(String,String)>,Vec<(char, [u8; 4])>) = serde_json::from_str(data).ok()?;
    let mut rules: Vec<Rule> = vec![];
    for (condition,action) in parsed_rules {
        rules.push(Rule::new(&condition, &action));
    }
    let mut colors = BTreeMap::new();
    for (color_key,color_value) in parsed_colors {
        colors.insert(color_key, color_value);
    }
    Some((rules,colors))
}