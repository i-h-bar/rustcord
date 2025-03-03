use regex::Captures;

pub fn add_emoji(cap: &Captures) -> String {
    match &cap[0] {
        "{T}" => "<:tap:1341154606524403804>",
        "{G}" => "<:G_:1345438564242165920>",
        "{Q}" => "<:Q_:1345438796866392165>",
        "{E}" => "<:E_:1345438529609531502>",
        "{P}" => "<:P_:1345440710085709914>",
        "{PW}" => "<:PW:1345440736929120318>",
        "{CHAOS}" => "<:CHAOS:1345440488161017997>",
        "{A}" => "<:A_:1345440412441116693>",
        "{TK}" => "<:TK:1346103090968793150>",
        "{X}" => "<:X_:1345704187551547392>",
        "{Y}" => "<:Y_:1346103088972300398>",
        "{Z}" => "<:Z_:1345440514337406988>",
        "{0}" => "<:0_:1345437656942645308>",
        "{Â½}" => "<:1_2:1346103097059180556>",
        "{1}" => "<:1_:1345437700534304798>",
        "{2}" => "<:2_:1345437748453965916>",
        "{3}" => "<:3_:1345437898631286815>",
        "{4}" => "<:4_:1345437928624488478>",
        "{5}" => "<:5_:1345437962367930429>",
        "{6}" => "<:6_:1345437996123684884>",
        "{7}" => "<:7_:1345438032504950877>",
        "{8}" => "<:8_:1345438064079667271>",
        "{9}" => "<:9_:1345438095415443558>",
        "{10}" => "<:10:1345438121688436838>",
        "{11}" => "<:11:1345438143641288765>",
        "{12}" => "<:12:1345438162410799177>",
        "{13}" => "<:13:1345438183265009714>",
        "{14}" => "<:14:1345438202474926201>",
        "{15}" => "<:15:1345438222078967872>",
        "{16}" => "<:16:1345440302285979669>",
        "{17}" => "<:17:1345440324952133673>",
        "{18}" => "<:18:1345440347010105454>",
        "{19}" => "<:19:1345440370670178405>",
        "{20}" => "<:20:1345440391239045120>",
        "{100}" => "<:100_:1346103101207089153>",
        "{1000000}" => "<:1000000:1346103099508523119>",
        "{âˆž}" => "<:infinite:1346103098267009117>",
        "{W/U}" => "<:WU:1345702950651035658>",
        "{W/B}" => "<:WB:1345702946121191444>",
        "{B/R}" => "<:BR:1345702940480114749>",
        "{B/G}" => "<:BG:1345702935706730526>",
        "{U/B}" => "<:UB:1345439329278890097>",
        "{U/R}" => "<:UR:1345439405136941187>",
        "{R/G}" => "<:RG:1345438831737966694>",
        "{R/W}" => "<:RW:1345438965162835999>",
        "{G/W}" => "<:GW:1345438678306132029>",
        "{G/U}" => "<:GU:1345438620185530488>",
        "{B/G/P}" => "<:BGP:1345702936856236062>",
        "{B/R/P}" => "<:BRP:1345702942044327936>",
        "{G/U/P}" => "<:GUP:1345438650879574158>",
        "{G/W/P}" => "<:GWP:1345438712489578626>",
        "{R/G/P}" => "<:RGP:1345438898771202201>",
        "{R/W/P}" => "<:RWP:1345439000072294581>",
        "{U/B/P}" => "<:UBP:1345439361071583333>",
        "{U/R/P}" => "<:URP:1345702944997376081>",
        "{W/B/P}" => "<:WBP:1345702947228745771>",
        "{W/U/P}" => "<:WUP:1345702953452834836>",
        "{C/W}" => "<:CW:1345705665418756187>",
        "{C/U}" => "<:CU:1345438494918705226>",
        "{C/B}" => "<:CB:1345705955521859676>",
        "{C/R}" => "<:CR:1345438462328836200>",
        "{C/G}" => "<:CG:1345706079820058624>",
        "{2/W}" => "<:2W:1345437870554615840>",
        "{2/U}" => "<:2U:1345437843564007506>",
        "{2/B}" => "<:2B:1345437778250567742>",
        "{2/R}" => "<:2R:1345437820637941832>",
        "{2/G}" => "<:2G:1345437799494713455>",
        "{H}" => "<:H_:1345440568158715944>",
        "{W/P}" => "<:WP:1345703065872764939>",
        "{U/P}" => "<:UP:1345439290267537590>",
        "{B/P}" => "<:BP:1345702938131300423>",
        "{R/P}" => "<:RP:1345438934158807110>",
        "{G/P}" => "<:GP:1345438592872349819>",
        "{C/P}" => "<:CP:1345702943554535525>",
        "{HW}" => "<:HW:1345440643110928485>",
        "{HR}" => "<:HR:1345440614073761882>",
        "{W}" => "<:W_:1345439215546269776>",
        "{U}" => "<:U_:1345439182209683568>",
        "{B}" => "<:B_:1345438265351602350>",
        "{R}" => "<:R_:1345438424160800892>",
        "{C}" => "<:C_:1345438302806999161>",
        "{S}" => "<:S_:1345439116061311110>",
        "{L}" => "<:L_:1345440671821074553>",
        "{D}" => "<:D_:1346103094655717428>",
        default => default,
    }
    .to_string()
}
