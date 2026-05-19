#![allow(clippy::unreadable_literal)]
use contracts::card::Card;
use regex::Captures;
use serenity::all::{EmojiId, ReactionType};

const MANA_W: u64      = 1376146180525789234;
const MANA_U: u64      = 1376146058744303756;
const MANA_B: u64      = 1375912886932865136;
const MANA_R: u64      = 1376145976808570960;
const MANA_G: u64      = 1375927262590931014;
const MANA_C: u64      = 1375913113299189871;

const MANA_WU: u64     = 1376146222556905502;
const MANA_WB: u64     = 1376146178797735946;
const MANA_RW: u64     = 1376145969946820690;
const MANA_GW: u64     = 1375927260212629654;
const MANA_UB: u64     = 1376146057355858031;
const MANA_UR: u64     = 1376146183529037825;
const MANA_GU: u64     = 1375927256471572601;
const MANA_BR: u64     = 1375912881316561026;
const MANA_BG: u64     = 1375912885615591504;
const MANA_RG: u64     = 1376145975009087569;

const MANA_WP: u64     = 1376146169771593778;
const MANA_UP: u64     = 1376146054600196116;
const MANA_BP: u64     = 1375912882604343408;
const MANA_RP: u64     = 1376145971771346954;
const MANA_GP: u64     = 1375927566363262976;
const MANA_CP: u64     = 1375913107360190555;

const MANA_WUP: u64    = 1376146220833177740;
const MANA_WBP: u64    = 1376146171365425222;
const MANA_UBP: u64    = 1376146055795703818;
const MANA_URP: u64    = 1376146182073749514;
const MANA_BRP: u64    = 1375912879881977906;
const MANA_BGP: u64    = 1375912884084805722;
const MANA_RGP: u64    = 1376145973448937674;
const MANA_RWP: u64    = 1376145967547682836;
const MANA_GWP: u64    = 1375914260474892319;
const MANA_GUP: u64    = 1376160251824574534;


fn custom_emoji(id: u64, name: &str) -> ReactionType {
    ReactionType::Custom {
        animated: false,
        id: EmojiId::new(id),
        name: Some(name.to_string()),
    }
}

pub fn colour_id_emoji(card: &Card) -> ReactionType {
    let identity = card.colour_identity().join("");

    if identity.is_empty() {
        return custom_emoji(MANA_C, "C_");
    }

    if card.mana_cost().contains("/P") {
        return match identity.as_str() {
            "W" => custom_emoji(MANA_WP, "W_P"),
            "U" => custom_emoji(MANA_UP, "U_P"),
            "B" => custom_emoji(MANA_BP, "B_P"),
            "R" => custom_emoji(MANA_RP, "R_P"),
            "G" => custom_emoji(MANA_GP, "G_P"),
            "C" => custom_emoji(MANA_CP, "C_P"),
            "WU" | "UW" => custom_emoji(MANA_WUP, "W_U_P"),
            "WB" | "BW" => custom_emoji(MANA_WBP, "W_B_P"),
            "UB" | "BU" => custom_emoji(MANA_UBP, "U_B_P"),
            "UR" | "RU" => custom_emoji(MANA_URP, "U_R_P"),
            "BR" | "RB" => custom_emoji(MANA_BRP, "B_R_P"),
            "BG" | "GB" => custom_emoji(MANA_BGP, "B_G_P"),
            "RG" | "GR" => custom_emoji(MANA_RGP, "R_G_P"),
            "RW" | "WR" => custom_emoji(MANA_RWP, "R_W_P"),
            "GW" | "WG" => custom_emoji(MANA_GWP, "G_W_P"),
            "GU" | "UG" => custom_emoji(MANA_GUP, "G_U_P"),
            s if s.len() == 5 => ReactionType::Unicode("🌈".to_string()),
            _ => ReactionType::Unicode("✨".to_string()),
        };
    }

    match identity.as_str() {
        // Single colours
        "W" => custom_emoji(MANA_W, "W_"),
        "U" => custom_emoji(MANA_U, "U_"),
        "B" => custom_emoji(MANA_B, "B_"),
        "R" => custom_emoji(MANA_R, "R_"),
        "G" => custom_emoji(MANA_G, "G_"),
        "C" => custom_emoji(MANA_C, "C_"),
        // Two colours (both orderings)
        "WU" | "UW" => custom_emoji(MANA_WU, "W_U"),
        "WB" | "BW" => custom_emoji(MANA_WB, "W_B"),
        "WR" | "RW" => custom_emoji(MANA_RW, "R_W"),
        "WG" | "GW" => custom_emoji(MANA_GW, "G_W"),
        "UB" | "BU" => custom_emoji(MANA_UB, "U_B"),
        "UR" | "RU" => custom_emoji(MANA_UR, "U_R"),
        "UG" | "GU" => custom_emoji(MANA_GU, "G_U"),
        "BR" | "RB" => custom_emoji(MANA_BR, "B_R"),
        "BG" | "GB" => custom_emoji(MANA_BG, "B_G"),
        "RG" | "GR" => custom_emoji(MANA_RG, "R_G"),
        s if s.len() == 5 => ReactionType::Unicode("🌈".to_string()),
        _ => ReactionType::Unicode("✨".to_string()),
    }
}

pub fn add_emoji(cap: &Captures) -> String {
    match &cap[0] {
        "{T}" => "<:T_:1376146062217056306>",
        "{G}" => "<:G_:1375927262590931014>",
        "{Q}" => "<:Q_:1376145872462544896>",
        "{E}" => "<:E_:1375914121534640168>",
        "{P}" => "<:P_:1376145875411271711>",
        "{PW}" => "<:PW:1376145874123620362>",
        "{CHAOS}" => "<:CHAOS:1375914124978163722>",
        "{A}" => "<:A_:1375914023425544352>",
        "{TK}" => "<:TK:1376146060258443405>",
        "{X}" => "<:X_:1376146219772018698>",
        "{Y}" => "<:Y_:1376146218073456661>",
        "{Z}" => "<:Z_:1376146216680816693>",
        "{0}" => "<:0_:1375912399290499124>",
        "{Â½}" => "<:__:1375914024948076627>",
        "{1}" => "<:1_:1375912395486269500>",
        "{2}" => "<:2_:1375912394324185239>",
        "{3}" => "<:3_:1375912525983383612>",
        "{4}" => "<:4_:1375912524570038353>",
        "{5}" => "<:5_:1375912522372223138>",
        "{6}" => "<:6_:1375912520803549290>",
        "{7}" => "<:7_:1375912519306182666>",
        "{8}" => "<:8_:1375912517842243736>",
        "{9}" => "<:9_:1375912516638478476>",
        "{10}" => "<:10:1375912515061420214>",
        "{11}" => "<:11:1375912723682164858>",
        "{12}" => "<:12:1375912653116936192>",
        "{13}" => "<:13:1375912651011526798>",
        "{14}" => "<:14:1375912649874866257>",
        "{15}" => "<:15:1375912648566116594>",
        "{16}" => "<:16:1375912647353962517>",
        "{17}" => "<:17:1375912645500080149>",
        "{18}" => "<:18:1375912643797319860>",
        "{19}" => "<:19:1375912642090107032>",
        "{20}" => "<:20:1375912640584355850>",
        "{100}" => "<:100:1375914027502403836>",
        "{1000000}" => "<:1000000:1375914026348970157>",
        "{âˆž}" => "<:infinite:1375912156368863233>",
        "{W/U}" => "<:W_U:1376146222556905502>",
        "{W/B}" => "<:W_B:1376146178797735946>",
        "{B/R}" => "<:B_R:1375912881316561026>",
        "{B/G}" => "<:B_G:1375912885615591504>",
        "{U/B}" => "<:U_B:1376146057355858031>",
        "{U/R}" => "<:U_R:1376146183529037825>",
        "{R/G}" => "<:R_G:1376145975009087569>",
        "{R/W}" => "<:R_W:1376145969946820690>",
        "{G/W}" => "<:G_W:1375927260212629654>",
        "{G/U}" => "<:G_U:1375927256471572601>",
        "{B/G/P}" => "<:B_G_P:1375912884084805722>",
        "{B/R/P}" => "<:B_R_P:1375912879881977906>",
        "{G/U/P}" => "<:G_U_P:1376160251824574534>",
        "{G/W/P}" => "<:G_W_P:1375914260474892319>",
        "{R/G/P}" => "<:R_G_P:1376145973448937674>",
        "{R/W/P}" => "<:R_W_P:1376145967547682836>",
        "{U/B/P}" => "<:U_B_P:1376146055795703818>",
        "{U/R/P}" => "<:U_R_P:1376146182073749514>",
        "{W/B/P}" => "<:W_B_P:1376146171365425222>",
        "{W/U/P}" => "<:W_U_P:1376146220833177740>",
        "{C/W}" => "<:C_W:1375913103140585533>",
        "{C/U}" => "<:C_U:1375913104533360660>",
        "{C/B}" => "<:C_B:1375913111420145866>",
        "{C/R}" => "<:C_R:1375913105992847540>",
        "{C/G}" => "<:C_G:1375913109574782996>",
        "{2/W}" => "<:2_W:1375912386837483580>",
        "{2/U}" => "<:2_U:1375912388670521394>",
        "{2/B}" => "<:2_B:1375912393154236577>",
        "{2/R}" => "<:2_R:1375912390373146654>",
        "{2/G}" => "<:2_G:1375912391807860819>",
        "{H}" => "<:H_:1376145883632107561>",
        "{W/P}" => "<:W_P:1376146169771593778>",
        "{U/P}" => "<:U_P:1376146054600196116>",
        "{B/P}" => "<:B_P:1375912882604343408>",
        "{R/P}" => "<:R_P:1376145971771346954>",
        "{G/P}" => "<:G_P:1375927566363262976>",
        "{C/P}" => "<:C_P:1375913107360190555>",
        "{HW}" => "<:HW:1376145880293314630>",
        "{HR}" => "<:HR:1376145882323353610>",
        "{W}" => "<:W_:1376146180525789234>",
        "{U}" => "<:U_:1376146058744303756>",
        "{B}" => "<:B_:1375912886932865136>",
        "{R}" => "<:R_:1376145976808570960>",
        "{C}" => "<:C_:1375913113299189871>",
        "{S}" => "<:S_:1376146064058617980>",
        "{L}" => "<:L_:1376145877772669050>",
        "{D}" => "<:D_:1375914122880749589>",
        default => default,
    }
    .to_string()
}
