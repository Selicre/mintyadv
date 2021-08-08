use std::path::Path;
use std::io;
use std::io::Write;
use std::fs::{self, File};
use std::env;
use std::collections::HashMap;

use image::GenericImageView;
use chrono::Datelike;

macro_rules! def_impl {
    ($name:ident) => {
        impl $name {
            fn write(&self, mut into: impl io::Write, name: &str) {
                writeln!(into, "pub const {}: {} = {:#X?};", name, stringify!($name), self).unwrap();
            }
            fn write_start(mut into: impl io::Write, name: &str, len: usize) {
                writeln!(into, "pub const {}: [{}; {}] = [", name, stringify!($name), len).unwrap();
            }
            fn write_entry(&self, mut into: impl io::Write) {
                writeln!(into, "    {:#X?},",self).unwrap();
            }
            fn write_end(mut into: impl io::Write) {
                writeln!(into, "];").unwrap();
            }
        }
    };
}

#[derive(Debug)]
struct DataDef {
    offset: usize,
    pal: usize
}

def_impl!(DataDef);

#[derive(Debug)]
struct LevelDef {
    offset: usize,
    width: u8,
    height: u8,
}

def_impl!(LevelDef);

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data.rs");
    let bin_path = Path::new(&out_dir).join("data.bin");
    let mut f = File::create(&dest_path).unwrap();

    let mut data = vec![];
    let mut pal = vec![];

    let img = image::open("assets/fg/fries.png").unwrap().to_rgba8();
    embed_fg(&img, 16, &mut data, &mut pal).write(&mut f, "BLOCKS");

    let img = image::open("assets/bg/hills.png").unwrap().to_rgba8();
    embed_bg(&img, &mut data, &mut pal).write(&mut f, "BG");

    let img = image::open("assets/sprites/toothpaste.png").unwrap().to_rgba8();
    embed_fg(&img, 32, &mut data, &mut pal).write(&mut f, "TOOTHPASTE");

    let img = image::open("assets/sprites/entities.png").unwrap().to_rgba8();
    embed_fg(&img, 32, &mut data, &mut pal).write(&mut f, "ENTITIES");

    /*
    let img = image::open("assets/sprites/particles.png").unwrap().to_rgba8();
    embed_fg(&img, 16, &mut data, &mut pal).write(&mut f, "PARTICLES");
    */

    let img = image::open("assets/fonts/boldface.png").unwrap().to_rgba8();
    embed_fg(&img, 8, &mut data, &mut pal).write(&mut f, "BOLDFACE");

    let mut ent_list = vec![];
    let mut ent = String::new();
    let mut ent_len = 0;

    let count = 2;

    writeln!(f, "pub static LEVEL_COUNT: usize = {};", count);

    LevelDef::write_start(&mut f, "MAPS", count);
    for i in 0..count {
        let level = serde_json::from_slice(&fs::read(format!("assets/maps/L0A{}.json", i)).unwrap()).unwrap();
        embed_map(&level, &mut data).write_entry(&mut f);
        ent_list.push(embed_entities(&level, &mut ent, &mut ent_len));
    }
    LevelDef::write_end(&mut f);

    ent_list.push(ent_len);

    write!(f, "pub static ENTITY_LIST: [EntityEntry; {}] = [{}];", ent_len, ent);
    write!(f, "pub static ENTITY_OFFSET: [usize; {}] = {:?};", ent_list.len(), ent_list);

    let comp = lz4::block::compress(&data, lz4::block::CompressionMode::HIGHCOMPRESSION(12).into(), false).unwrap();

    std::fs::write(bin_path, &comp).unwrap();
    std::fs::write("data.bin", &comp).unwrap();
    std::fs::write("data_raw.bin", &data).unwrap();

    writeln!(f, r#"pub static mut DATA: [u8; {0:}] = [0; {0:}];"#, data.len()).unwrap();
    writeln!(f, r#"pub static DATA_LZ4: [u8; {}] = *include_bytes!(concat!(env!("OUT_DIR"), "/data.bin"));"#, comp.len()).unwrap();
    writeln!(f, "pub static PAL_DATA: [u32; {}] = {:#08X?};", pal.len(), pal).unwrap();

    let title_path = Path::new(&out_dir).join("title.txt");
    let date = chrono::offset::Local::today();
    let month = date.month() as i32 + (date.year() - 2021) * 12;
    std::fs::write(title_path, format!("MINTY ADVENTURES BUILD {:02}{:02}", month, date.day()));

}

fn embed_map(level: &serde_json::Value, data: &mut Vec<u8>) -> LevelDef {
    let offset = data.len();
    let tiles = &level["layers"][0]["data"];
    for i in tiles.as_array().unwrap().iter() {
        data.push((i.as_u64().unwrap() - 1) as u8);
    }
    let width = level["width"].as_u64().unwrap() as u8;
    let height = level["height"].as_u64().unwrap() as u8;
    LevelDef {
        offset,
        width,
        height
    }
}

fn embed_entities(level: &serde_json::Value, out: &mut String, len: &mut usize) -> usize {
    use std::fmt::Write;
    let base = *len;
    let entities = &level["layers"][1]["objects"].as_array().unwrap();
    for i in entities.iter() {
        // x:u16 y:u16 id
        let x = (i["x"].as_u64().unwrap() / 0x10) as u8 + 1;
        let y = (i["y"].as_u64().unwrap() / 0x10) as u8 - 1;
        let id = match i["gid"].as_u64().unwrap() & 0xFFF {
            0x101 => 2,
            //0x45 => 3,
            //0x49 => 4,
            //0x51 => 5,
            _ => continue,
            //c => panic!("no such EntityKind: {}", c)
        };
        writeln!(out, r"EntityEntry {{
            x: {:#02X}, y: {:#02X}, kind: {}
        }}, ", x, y, id);
        *len += 1;
    }
    base
}

fn embed_fg(image: &image::RgbaImage, size: u32, data: &mut Vec<u8>, pal: &mut Vec<u32>) -> DataDef {
    let mut palette = HashMap::new();
    let offset = data.len();
    let pal_offset = pal.len();
    for ty in 0..image.height()/size {
        for tx in 0..image.width()/size {
            data.extend(image.view(tx*size, ty*size, size, size).pixels().map(|(_,_,c)| {
                let c = u32::from_le_bytes(c.0);
                let len = palette.len();
                let id = palette.entry(c).or_insert_with(|| {
                    pal.push(c);
                    len
                });
                *id as u8
            }));
        }
    }
    DataDef {
        offset,
        pal: pal_offset,
    }
}


fn embed_bg(image: &image::RgbaImage, data: &mut Vec<u8>, pal: &mut Vec<u32>) -> DataDef {
    let mut palette = HashMap::new();
    let offset = data.len();
    let pal_offset = pal.len();
    data.extend(image.pixels().map(|c| {
        let c = u32::from_le_bytes(c.0);
        let len = palette.len();
        let id = palette.entry(c).or_insert_with(|| {
            pal.push(c);
            len
        });
        *id as u8
    }));
    DataDef {
        offset,
        pal: pal_offset,
    }
}
