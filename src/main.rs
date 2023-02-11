use tuple::Map;
use std::fs::File;
use json::{
    self,
    JsonValue,
};
use std::io::{
    self,
    prelude::*,
};

#[derive(Copy, Clone)]
enum Finger {  // Const values are used to cast variants as usize
    LeftPinky   = 0,
    LeftRing    = 1,
    LeftMiddle  = 2,
    LeftIndex   = 3,
    RightIndex  = 4,
    RightMiddle = 5,
    RightRing   = 6,
    RightPinky  = 7,
    Thumb       = 8,
}

impl Finger {

    // Alias in case the const value thing is too counter intuitive
    #[allow(dead_code)]
    fn as_usize(&self) -> usize {
        *self as usize
    }

    fn from_scan_code(input: &str) -> Option<Self> {
        match input {
            "Space" => Some(Self::Thumb),
            "Digit1" | "KeyQ" | "KeyA" | "KeyZ" | "IntlBackslash" => Some(Self::LeftPinky),
            "Digit2" | "KeyW" | "KeyS" | "KeyX"   => Some(Self::LeftRing),
            "Digit3" | "KeyE" | "KeyD" | "KeyC"   => Some(Self::LeftMiddle),
            "Digit4" | "KeyR" | "KeyF" | "KeyV" |
            "Digit5" | "KeyT" | "KeyG" | "KeyB"   => Some(Self::LeftIndex),
            "Digit6" | "KeyY" | "KeyH" | "KeyN" |
            "Digit7" | "KeyU" | "KeyJ" | "KeyM"   => Some(Self::RightIndex),
            "Digit8" | "KeyI" | "KeyK" | "Comma"  => Some(Self::RightMiddle),
            "Digit9" | "KeyO" | "KeyL" | "Period" => Some(Self::RightRing),
            "Digit0" | "KeyP" | "Semicolon" | "Slash" |
            "Minus" | "Equal" | "BracketLeft" | "BracketRight" |
            "Quote" | "Backquote" | "Backslash"   => Some(Self::RightPinky),
            _ => None,
        }
    }

}

enum Fingering {
    Unigram(Finger),
    Bigram(Finger, Finger),
}

impl Fingering {

    fn from_symbol_in_layout(layout: &JsonValue, symbol: &str) -> Option<Self> {
        let get_finger_in_keymap = |layout: &JsonValue, symbol: &str| -> Option<Finger> {
            layout["keymap"].entries()
                .find(|(_key, symbol_list)| symbol_list.contains(symbol))
                .and_then(|(key, _symbol_list)| Finger::from_scan_code(key))
        };

        let get_bigram_using_dead_keys = |layout: &JsonValue, diacritic: &str| -> Option<Self> {

            // Returns a String ’cause I couldn’t specify lifetime anotations.
            let find_primitive = |dead_key_map: &JsonValue, diacritic: &str| -> Option<String> {
                dead_key_map.entries()
                    .find(|(_primitive, dia)| diacritic == dia.as_str().unwrap())
                    .map(|(primitive, _dia)| primitive.to_owned())
            };

            layout["deadkeys"].entries()
                .find_map(|(dead_key, map)| Some(dead_key).zip(find_primitive(map, diacritic)))
                .map(|(dk, pri)| (dk, pri.as_str()).map(|sym| get_finger_in_keymap(layout, sym)))
                // .and_then(|(dk, pri)| dk.zip_with(pri, Self::Bigram))
                .and_then(|(dk, pri)| dk.zip(pri))  // can’t use zip_with yet, as it’s not stable
                .map(|(dk, pri)| Self::Bigram(dk, pri))
        };

        get_finger_in_keymap(layout, symbol).map(Self::Unigram)
            .or_else(|| get_bigram_using_dead_keys(layout, symbol))
    }

}

fn show_finger_data(layout: &JsonValue, corpus: &JsonValue) -> () {
    let mut frequency_per_finger: [f64; 9] = [0.; 9];
    let mut ignored: f64 = 0.;
    let mut excess_frequency: f64 = 0.;

    for (symbol, frequency) in corpus["symbols"].entries() {
        let frequency = frequency.as_f64().unwrap();

        let Some(fingering) = Fingering::from_symbol_in_layout(layout, symbol) else {
            ignored += frequency;
            continue;
        };

        match fingering {
            Fingering::Unigram(finger) => frequency_per_finger[finger as usize] += frequency,
            Fingering::Bigram(finger1, finger2) => {
                excess_frequency += frequency;
                frequency_per_finger[finger1 as usize] += frequency;
                frequency_per_finger[finger2 as usize] += frequency;
            },
        };
    };

    // Normalise data to compensate dead keys doubling the frequecy of some symbols
    frequency_per_finger = frequency_per_finger.map(|x| x * 100. / (excess_frequency + 100.));

    println!("Finger frequency : {:#?}", frequency_per_finger);
    println!("Percent of ignored symbols : {ignored}");
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args[1].as_str() == "-h" || args[1].as_str() == "--help" {
        println!("Usage: {} <layout.json> <corpus.json>", args[0]);
        return Ok(());
    } 
    
    let open_json_file = |file_name: &str| -> io::Result<JsonValue> {
        let mut file_contents = String::new();
        File::open(file_name)?.read_to_string(&mut file_contents)?;
        Ok(json::parse(file_contents.as_str()).unwrap())
    };

    let layout = open_json_file(&args[1])?;
    let corpus = open_json_file(&args[2])?;

    show_finger_data(&layout, &corpus);

    Ok(())
}
