use serenity::model::Colour;

pub fn get_colour_identity(colour_id: Vec<String>) -> Colour {
    let (mut r, mut g, mut b) = (0, 0, 0);
    let len = colour_id.len() as f32;
    for colour in colour_id.iter() {
        let (red, green, blue) = get_colour_num(colour);
        r += red;
        g += green;
        b += blue;
    }

    let r = (r as f32 / len).floor() as u8;
    let g = (g as f32 / len).floor() as u8;
    let b = (b as f32 / len).floor() as u8;
    log::info!("{},{},{}", r, g, b);
    Colour::from_rgb(r, g, b)
}

fn get_colour_num(colour: &str) -> (u16, u16, u16) {
    match colour {
        "R" => { (255, 0, 0) }
        "G" => { (34, 139, 34) }
        "U" => { (0, 0, 255) }
        "W" => { (255, 255, 255) }
        "B" => { (0, 0, 0) }
        _ => { (0, 0, 0) }
    }
}