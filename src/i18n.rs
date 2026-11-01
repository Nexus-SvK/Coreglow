//! Interface translations. English is the source text; Slovak is used when the
//! system language is Slovak (LC_ALL / LC_MESSAGES / LANG starting with "sk").
//! COREGLOW_LANG=en or =sk overrides it. Texts with values use "{}" placeholders.

use std::fmt::Display;
use std::sync::OnceLock;

static SK: &[(&str, &str)] = &[
    // quick buttons, effects
    ("Lights off", "Vypnúť svetlá"),
    ("Quiet fans", "Tiché ventilátory"),
    ("All white", "Všetko biele"),
    ("Rainbow", "Dúha"),
    ("Solid colour", "Jedna farba"),
    ("One colour, no movement", "Jedna farba bez pohybu"),
    ("Breathing", "Dýchanie"),
    ("Slowly fades in and out", "Pomaly sa rozsvecuje a stmieva"),
    ("Flashing", "Blikanie"),
    ("Blinks on and off", "Bliká"),
    ("Colour cycle", "Striedanie farieb"),
    ("Fades through your colours", "Plynulo prechádza vašimi farbami"),
    ("Everything changes colour together", "Všetko mení farbu naraz"),
    ("Rainbow wave", "Dúhová vlna"),
    ("A rainbow moving along the lights", "Dúha putujúca po svetlách"),
    ("Two-colour wave", "Dvojfarebná vlna"),
    ("Two colours moving along the lights", "Dve farby putujúce po svetlách"),
    ("Shows temperature", "Ukazuje teplotu"),
    ("Colour changes as it gets hotter", "Farba sa mení s teplotou"),
    ("Off", "Vypnuté"),
    // saving
    ("Saving…", "Ukladá sa…"),
    ("Saved", "Uložené"),
    ("Not saved: {}", "Neuložené: {}"),
    ("collecting…", "zbierajú sa údaje…"),
    // colours
    ("White", "Biela"),
    ("Warm white", "Teplá biela"),
    ("Red", "Červená"),
    ("Orange", "Oranžová"),
    ("Green", "Zelená"),
    ("Blue", "Modrá"),
    ("Purple", "Fialová"),
    ("custom", "vlastná"),
    ("recent", "nedávne"),
    // parts
    ("Left fan", "Ľavý ventilátor"),
    ("Right fan", "Pravý ventilátor"),
    ("Pump", "Pumpa"),
    ("Pump head", "Hlava pumpy"),
    ("LED strips 1–4", "LED pásy 1–4"),
    ("LED strip 1", "LED pás 1"),
    ("LED strip 2", "LED pás 2"),
    ("LED strip 3", "LED pás 3"),
    ("LED strip 4", "LED pás 4"),
    ("All LED strips", "Všetky LED pásy"),
    ("Both radiator fans", "Oba ventilátory radiátora"),
    ("Motherboard (via OpenRGB)", "Základná doska (cez OpenRGB)"),
    ("Everything", "Všetko"),
    ("{}: click to change its lighting", "{}: kliknutím zmeníte osvetlenie"),
    // effect settings
    ("Colour", "Farba"),
    ("Colours", "Farby"),
    ("+ add", "+ pridať"),
    ("− remove", "− odobrať"),
    ("Speed", "Rýchlosť"),
    ("slow", "pomaly"),
    ("fast", "rýchlo"),
    ("Wave length", "Dĺžka vlny"),
    ("long", "dlhá"),
    ("short", "krátka"),
    ("Brightness", "Jas"),
    ("Follows", "Riadi sa"),
    ("Cool", "Chladné"),
    ("Hot", "Horúce"),
    ("Warm", "Teplé"),
    ("remove", "odobrať"),
    ("+ add a colour", "+ pridať farbu"),
    ("Warning", "Upozornenie"),
    ("Flash when it gets too hot", "Blikať pri príliš vysokej teplote"),
    ("above", "nad"),
    // fans
    ("last 10 min: {}–{} °C", "posledných 10 min: {}–{} °C"),
    ("Follow temperature", "Podľa teploty"),
    ("Fixed speed", "Pevná rýchlosť"),
    ("Quick choice", "Rýchla voľba"),
    ("Quiet", "Tichý"),
    ("Balanced", "Vyvážený"),
    ("Performance", "Výkonný"),
    ("Full speed", "Plný výkon"),
    (
        "Drag a dot to change it · double-click to add · right-click to remove · click a dot, then use the arrow keys for fine steps",
        "Bod presuniete ťahaním · dvojklik pridá bod · pravý klik ho odstráni · kliknite na bod a dolaďte ho šípkami",
    ),
    // sidebar, pages
    ("Home", "Domov"),
    ("Lighting", "Osvetlenie"),
    ("Fans", "Ventilátory"),
    ("Profiles", "Profily"),
    ("Service running", "Služba beží"),
    ("Service isn't running", "Služba nebeží"),
    ("Start it", "Spustiť"),
    ("· profile “{}”", "· profil „{}“"),
    ("Automatic switching: right now “{}” is in use because {}.", "Automatické prepínanie: teraz sa používa „{}“, pretože {}."),
    // home
    ("Your PC", "Váš počítač"),
    ("There's a problem — see the sidebar.", "Vyskytol sa problém — pozrite bočný panel."),
    ("Everything is running normally.", "Všetko beží normálne."),
    ("The lighting service isn't running.", "Služba osvetlenia nebeží."),
    ("Profile", "Profil"),
    ("CPU", "Procesor"),
    ("last {} minutes", "posledných {} minút"),
    ("Inside the case", "V skrinke"),
    ("probe 2: {}", "sonda 2: {}"),
    ("Radiator fans", "Ventilátory radiátora"),
    ("Right fan  {} rpm", "Pravý ventilátor  {} ot./min"),
    ("Left fan   {} rpm", "Ľavý ventilátor   {} ot./min"),
    ("Full speed while the lighting is controlled from the PC. That's normal.", "Kým osvetlenie riadi počítač, beží naplno. To je normálne."),
    ("Your PC right now", "Váš počítač práve teraz"),
    ("Click a part to change its lighting", "Kliknutím na časť zmeníte jej osvetlenie"),
    ("Quick changes", "Rýchle zmeny"),
    ("Make a profile called “{}” to use this", "Na použitie vytvorte profil s názvom „{}“"),
    // lighting page
    ("What to change", "Čo zmeniť"),
    ("Whole PC", "Celý počítač"),
    ("Or give one part its own lighting:", "Alebo dajte jednej časti vlastné osvetlenie:"),
    ("own", "vlastné"),
    ("same", "rovnaké"),
    ("Include the motherboard", "Zahrnúť základnú dosku"),
    ("Sends the same effects to the motherboard lighting through the OpenRGB server", "Posiela rovnaké efekty na osvetlenie základnej dosky cez server OpenRGB"),
    ("Uses OpenRGB's server (openrgb --server), which is already running here.", "Používa server OpenRGB (openrgb --server), ktorý tu už beží."),
    ("Connecting…", "Pripája sa…"),
    ("Effect for the whole PC", "Efekt pre celý počítač"),
    ("Same as the whole PC", "Ako celý počítač"),
    ("Its own lighting", "Vlastné osvetlenie"),
    (
        "If two choices cover the same part (for example “Both fans” and “Left fan”), the one lower in the list wins.",
        "Ak dve voľby pokrývajú tú istú časť (napríklad „Oba ventilátory“ a „Ľavý ventilátor“), platí tá nižšie v zozname.",
    ),
    ("This part follows the whole-PC lighting.", "Táto časť sa riadi osvetlením celého počítača."),
    // fans page
    ("Fan speed", "Otáčky ventilátorov"),
    ("This profile also has separate settings for one fan, set in the profiles file.", "Tento profil má aj samostatné nastavenie pre jeden ventilátor (v súbore profilov)."),
    ("Right now", "Práve teraz"),
    ("Right fan   {} rpm · {} %", "Pravý ventilátor   {} ot./min · {} %"),
    ("Left fan    {} rpm · {} %", "Ľavý ventilátor    {} ot./min · {} %"),
    ("right fan, last 10 minutes", "pravý ventilátor, posledných 10 minút"),
    ("Testing: fans at full speed…", "Test: ventilátory naplno…"),
    ("Test fans (full speed for 5 s)", "Otestovať ventilátory (5 s naplno)"),
    ("The lighting service isn't running", "Služba osvetlenia nebeží"),
    ("{} rpm", "{} ot./min"),
    (
        "Runs at full speed while the lighting is controlled from the PC. That keeps the CPU cool and is normal.",
        "Kým osvetlenie riadi počítač, beží naplno. Udržiava procesor chladný a je to normálne.",
    ),
    ("When Coreglow is closed", "Keď je Coreglow zatvorený"),
    (
        "The controller keeps its own backup curve. It only reads case probe 1, and the pump always stays at full speed. Changes are stored in the controller within a second.",
        "Ovládač má vlastnú záložnú krivku. Číta len sondu 1 v skrinke a pumpa vždy beží naplno. Zmeny sa do ovládača uložia do sekundy.",
    ),
    ("Edit backup curve", "Upraviť záložnú krivku"),
    ("{} of {} points", "{} z {} bodov"),
    ("Reset to default", "Obnoviť predvolené"),
    // profiles page
    ("+ New profile", "+ Nový profil"),
    ("My profile", "Môj profil"),
    ("A profile is a saved set of colours and fan speeds. Click one to edit it.", "Profil je uložená sada farieb a otáčok ventilátorov. Kliknutím ho upravíte."),
    ("in use", "používa sa"),
    ("Fans: fixed {} %", "Ventilátory: pevne {} %"),
    ("Fans: follow {}", "Ventilátory: podľa {}"),
    ("Name", "Názov"),
    ("Use this profile", "Použiť tento profil"),
    ("Edit lighting", "Upraviť osvetlenie"),
    ("Edit fans", "Upraviť ventilátory"),
    ("Duplicate", "Duplikovať"),
    ("Click again to delete", "Kliknite znova na zmazanie"),
    ("Delete", "Zmazať"),
    ("Switch to another profile first", "Najprv prepnite na iný profil"),
    (
        "You're editing a profile that isn't in use. Click “Use this profile” to see it on your PC.",
        "Upravujete profil, ktorý sa nepoužíva. Kliknite na „Použiť tento profil“, aby ste ho videli na počítači.",
    ),
    ("Switch automatically", "Prepínať automaticky"),
    ("On", "Zapnuté"),
    ("Rules are checked from top to bottom; the first one that matches picks the profile.", "Pravidlá sa kontrolujú zhora nadol; profil určí prvé, ktoré platí."),
    ("When", "Keď"),
    ("part of its name, e.g. steamapps", "časť jeho názvu, napr. steamapps"),
    ("24-hour time, HH:MM", "24-hodinový čas, HH:MM"),
    ("from", "od"),
    ("to", "do"),
    ("use", "použiť"),
    ("Up", "Hore"),
    ("Down", "Dole"),
    ("Remove", "Odstrániť"),
    ("+ Add a rule", "+ Pridať pravidlo"),
    ("Otherwise: “{}”, the profile you chose.", "Inak: „{}“, profil, ktorý ste vybrali."),
    (
        "Tip: Steam games run from a folder called “steamapps”, so a “program is running” rule with that name catches all of them.",
        "Tip: hry zo Steamu bežia z priečinka „steamapps“, takže pravidlo „beží program“ s týmto názvom zachytí všetky.",
    ),
    ("The profiles file couldn't be read", "Súbor profilov sa nedal načítať"),
    ("Fix or delete {} and reopen this window.", "Opravte alebo zmažte {} a znova otvorte toto okno."),
    ("Coreglow — PC Lighting & Fans", "Coreglow — osvetlenie a ventilátory"),
    // rules
    ("A temperature is above", "Teplota je nad"),
    ("A program is running", "Beží program"),
    ("During a time of day", "V určitom čase dňa"),
    ("The screen is locked", "Obrazovka je zamknutá"),
    ("{} is above {} °C", "{} je nad {} °C"),
    ("a program matching “{}” is running", "beží program zodpovedajúci „{}“"),
    ("it's between {} and {}", "je medzi {} a {}"),
    ("the screen is locked", "obrazovka je zamknutá"),
    // sensors
    ("Processor (CPU)", "Procesor (CPU)"),
    ("Case probe 1", "Sonda v skrinke 1"),
    ("Case probe 2", "Sonda v skrinke 2"),
    ("Built-in graphics", "Integrovaná grafika"),
    ("Graphics card (RTX 3090)", "Grafická karta (RTX 3090)"),
    ("{} — needs NVIDIA driver", "{} — potrebuje ovládač NVIDIA"),
    // tray
    ("Lighting service isn't running", "Služba osvetlenia nebeží"),
    ("CPU {} °C · ", "Procesor {} °C · "),
    ("fans {}/{} rpm · pump {} rpm", "ventilátory {}/{} ot./min · pumpa {} ot./min"),
    ("Automatic: “{}” because {}", "Automaticky: „{}“, pretože {}"),
    ("Switch profiles automatically", "Prepínať profily automaticky"),
    ("Open Coreglow", "Otvoriť Coreglow"),
    ("Hide this icon", "Skryť túto ikonu"),
    // layout and devices
    ("External RGB port", "Externý RGB port"),
    ("RGB port {}", "RGB port {}"),
    ("External device {}", "Externé zariadenie {}"),
    ("What's connected", "Čo je pripojené"),
    ("The lighting service reports this when it starts.", "Služba osvetlenia to zistí pri spustení."),
    ("detected: {} ({} LEDs)", "zistené: {} ({} LED)"),
    ("detected: {} LEDs", "zistené: {} LED"),
    ("not auto-detected", "nezisťuje sa automaticky"),
    ("nothing detected", "nič nezistené"),
    ("A different build can describe its ports as [layout] in the profiles file (see man coreglowctl).", "Iná zostava môže opísať svoje porty ako [layout] v súbore profilov (pozri man coreglowctl)."),
    // 0.8: Commander CORE, night dimming
    ("All fans", "Všetky ventilátory"),
    ("Liquid (pump)", "Kvapalina (pumpa)"),
    ("Fan 1   {} rpm · {} %", "Ventilátor 1   {} ot./min · {} %"),
    ("Fan 2   {} rpm · {} %", "Ventilátor 2   {} ot./min · {} %"),
    ("fan 1, last 10 minutes", "ventilátor 1, posledných 10 minút"),
    ("Lower is quieter. It never goes below 50 %, so the CPU stays cool.", "Nižšia je tichšia. Nikdy neklesne pod 50 %, aby procesor zostal chladný."),
    ("The controller keeps its own backup curve. It only reads the liquid temperature, and the pump always stays at full speed. Changes are stored in the controller within a second.", "Ovládač si drží vlastnú záložnú krivku. Číta len teplotu kvapaliny a pumpa vždy beží naplno. Zmeny sa do ovládača uložia do sekundy."),
    ("Support for this controller is new and untested. If something looks wrong, please report it.", "Podpora tohto ovládača je nová a zatiaľ neodskúšaná. Ak niečo nevyzerá správne, dajte nám vedieť."),
    ("Dim at night", "Stlmiť v noci"),
    ("Dim all lighting at night", "V noci stlmiť všetko osvetlenie"),
    ("From", "Od"),
    ("To", "Do"),
    ("Dimmed right now.", "Práve je stlmené."),
    ("Drag the times, or click and type HH:MM.", "Časy posuňte myšou, alebo kliknite a napíšte HH:MM."),
    // 0.9: layout editor
    ("Change what's connected…", "Zmeniť, čo je pripojené…"),
    ("{}: devices on each RGB port, in the order they're chained. The name is what the Lighting page calls the part.", "{}: zariadenia na každom RGB porte v poradí, v akom sú zapojené. Názov je to, ako časť volá stránka Osvetlenie."),
    ("Add…", "Pridať…"),
    ("controller sees {} LEDs{}", "ovládač vidí {} LED{}"),
    ("Other parts from the profiles file:", "Ďalšie časti zo súboru profilov:"),
    ("“{}” can't be used as a name here", "„{}“ sa tu nedá použiť ako názov"),
    ("Profiles lose their own lighting for: {}", "Profily prídu o vlastné osvetlenie pre: {}"),
    ("Apply", "Použiť"),
    ("Cancel", "Zrušiť"),
    ("Use what the controller detects", "Použiť, čo zistí ovládač"),
    ("Back to the built-in layout", "Späť na vstavané zapojenie"),
    ("Removes [layout] from the profiles file", "Odstráni [layout] zo súboru profilov"),
];fn slovak() -> bool {
    static SLOVAK: OnceLock<bool> = OnceLock::new();
    *SLOVAK.get_or_init(|| {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let lang = var("COREGLOW_LANG").or_else(|| var("LC_ALL")).or_else(|| var("LC_MESSAGES")).or_else(|| var("LANG")).unwrap_or_default();
        lang.starts_with("sk")
    })
}

/// The text in the interface language.
pub fn tr(s: &'static str) -> &'static str {
    if !slovak() {
        return s;
    }
    SK.iter().find(|(en, _)| *en == s).map_or(s, |(_, sk)| sk)
}

/// Translate a template and fill its "{}" placeholders in order.
pub fn trf(template: &'static str, args: &[&dyn Display]) -> String {
    let mut out = String::new();
    let mut rest = tr(template);
    for a in args {
        match rest.split_once("{}") {
            Some((before, after)) => {
                out.push_str(before);
                out.push_str(&a.to_string());
                rest = after;
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_fill_in_order_and_slovak_keeps_placeholders() {
        assert_eq!(trf("probe 2: {}", &[&"26 °C"]), if slovak() { "sonda 2: 26 °C" } else { "probe 2: 26 °C" });
        for (en, sk) in SK {
            assert_eq!(en.matches("{}").count(), sk.matches("{}").count(), "placeholders differ: {en}");
        }
        let mut keys: Vec<&str> = SK.iter().map(|(en, _)| *en).collect();
        keys.sort();
        let n = keys.len();
        keys.dedup();
        assert_eq!(n, keys.len(), "duplicate English text in the table");
    }
}
